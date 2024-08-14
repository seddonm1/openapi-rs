use std::ops::Not;

use dropshot::{HttpError, RequestContext};
use kratos::apis::{configuration::Configuration, frontend_api::ToSessionError};
use uuid::Uuid;

use crate::{
    context,
    entity::{identity_user::IdentityUser, user::User},
};

#[derive(Debug, Clone)]
pub struct Kratos {
    pub public_configuration: Configuration,
    pub admin_configuration: Configuration,
}

impl Kratos {
    pub fn new(public_port: u16, admin_port: u16) -> Self {
        let public_configuration = kratos::apis::configuration::Configuration {
            base_path: format!("http://localhost:{}", public_port),
            ..Default::default()
        };

        let admin_configuration = kratos::apis::configuration::Configuration {
            base_path: format!("http://localhost:{}", admin_port),
            ..Default::default()
        };
        Self {
            public_configuration,
            admin_configuration,
        }
    }
}

impl User {
    /// Try to create a User from a request context.
    ///
    /// This function attempts to retrieve the user from the request context, falling back to creating a new user if not found.
    ///
    /// The `X-Session-Token` header is used to authenticate the request and verify the user's session.
    pub async fn try_from(rqctx: &RequestContext<context::Context>) -> Result<Self, HttpError> {
        // Check if the `X-Session-Token` header is present and can be decoded to a string.
        let session_token = match rqctx
            .request
            .headers()
            .get("X-Session-Token")
            .map(|v| v.to_str())
        {
            Some(Ok(session_token)) => session_token,
            _ => return Err(HttpError::for_status(None, http::StatusCode::UNAUTHORIZED)),
        };

        // Use the Kratos API to resolve the session associated with the `session_token`.
        let session = match kratos::apis::frontend_api::to_session(
            &rqctx.context().kratos().public_configuration,
            Some(session_token),
            None,
            None,
        )
        .await
        {
            Ok(session) => session,
            // If a response error is encountered return the correct error type.
            Err(kratos::apis::Error::ResponseError(err)) => {
                match err.entity {
                    Some(to_session_error) => match to_session_error {
                        // This should be the most common error.
                        ToSessionError::Status401(_) => {
                            return Err(HttpError::for_status(None, http::StatusCode::UNAUTHORIZED))
                        }
                        ToSessionError::Status403(_) => {
                            return Err(HttpError::for_status(None, http::StatusCode::UNAUTHORIZED))
                        }
                        ToSessionError::DefaultResponse(err) => {
                            return Err(HttpError::for_internal_error(format!("{:?}", err.error)))
                        }
                        ToSessionError::UnknownValue(err) => {
                            return Err(HttpError::for_internal_error(err.to_string()))
                        }
                    },
                    None => return Err(HttpError::for_internal_error(format!("{:?}", err))),
                };
            }
            // For any other error, return an internal server error with the error message.
            Err(err) => return Err(HttpError::for_internal_error(err.to_string())),
        };

        // Verify that the session is active.
        if session.active.unwrap_or(false).not() {
            return Err(HttpError::for_status(None, http::StatusCode::UNAUTHORIZED));
        };

        // Verify that the Session contains an Identity.
        let identity = match &session.identity {
            Some(identity) => identity,
            _ => return Err(HttpError::for_status(None, http::StatusCode::UNAUTHORIZED)),
        };

        // Verify that Identity contains an UUID id.
        let identity_id = match Uuid::parse_str(&identity.id) {
            Ok(id) => id,
            Err(_) => return Err(HttpError::for_status(None, http::StatusCode::UNAUTHORIZED)),
        };

        // If the IdentityUser already exists then retrieve the User.
        // Otherwise Upsert a new User. This is safe because the validity of the token has been
        // guaranteed by the Kratos instance.
        let identity_id_move = identity_id.clone();
        let user = match rqctx
            .context()
            .database()
            .read(move |connection| {
                let txn = connection.transaction()?;
                Ok(IdentityUser::retrieve(&txn, &identity_id_move)?
                    .map(|identity_user| identity_user.retrieve_user(&txn))
                    .transpose()?)
            })
            .await?
        {
            Some(user) => user,
            None => {
                rqctx
                    .context()
                    .database()
                    .write(move |connection| {
                        let txn = connection.transaction()?;
                        let user = User::new(Uuid::new_v4());
                        user.upsert(&txn)?;
                        user.create_identity_user(&txn, &identity_id_move)?;
                        txn.commit()?;
                        Ok(user)
                    })
                    .await?
            }
        };

        // If all checks pass, return the User.
        Ok(user)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use kratos::models::{
        CreateIdentityBody, Identity, IdentityWithCredentials, IdentityWithCredentialsPassword,
        IdentityWithCredentialsPasswordConfig, SuccessfulNativeLogin, VerifiableIdentityAddress,
    };
    use serde_json::json;

    impl super::Kratos {
        pub async fn create_user(
            &self,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Result<Identity> {
            let email = email.into();
            let password = password.into();

            // create test_user
            Ok(kratos::apis::identity_api::create_identity(
                &self.admin_configuration,
                Some(CreateIdentityBody {
                    credentials: Some(
                        IdentityWithCredentials {
                            oidc: None,
                            password: Some(
                                IdentityWithCredentialsPassword {
                                    config: Some(
                                        IdentityWithCredentialsPasswordConfig {
                                            hashed_password: None,
                                            password: Some(password),
                                        }
                                        .into(),
                                    ),
                                }
                                .into(),
                            ),
                        }
                        .into(),
                    ),
                    metadata_admin: None,
                    metadata_public: None,
                    recovery_addresses: None,
                    schema_id: "default".to_string(),
                    state: Some(kratos::models::create_identity_body::State::Active),
                    traits: json!({"email": email.clone()}),
                    verifiable_addresses: Some(vec![VerifiableIdentityAddress {
                        created_at: None,
                        id: None,
                        status: "active".to_string(),
                        updated_at: None,
                        value: email.clone(),
                        verified: true,
                        verified_at: None,
                        via: kratos::models::verifiable_identity_address::Via::Email,
                    }]),
                }),
            )
            .await?)
        }

        pub async fn login(
            &self,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Result<SuccessfulNativeLogin> {
            let email = email.into();
            let password = password.into();

            let login_flow = kratos::apis::frontend_api::create_native_login_flow(
                &self.public_configuration,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await?;

            Ok(kratos::apis::frontend_api::update_login_flow(
                &self.public_configuration,
                &login_flow.id,
                kratos::models::UpdateLoginFlowBody::Password(
                    kratos::models::UpdateLoginFlowWithPasswordMethod::new(
                        email,
                        "password".to_string(),
                        password,
                    )
                    .into(),
                ),
                None,
                None,
            )
            .await?)
        }
    }
}
