use crate::{api::ServerApi, entity::user::User};
use anyhow::Result;
use dropshot::{HttpError, HttpResponseOk, RequestContext};

pub(crate) enum ServerImpl {}

impl ServerApi for ServerImpl {
    type Context = crate::context::Context;

    #[doc = " Get the value of the counter."]
    async fn get_user(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseOk<User>, HttpError> {
        Ok(HttpResponseOk(User::try_from(&rqctx).await?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::TestContext;
    use http::HeaderMap;
    use uuid::Uuid;

    #[tokio::test]
    pub async fn get_user() -> Result<()> {
        let context = TestContext::new(vec![]).await?;

        let user = User::new(Uuid::new_v4());

        let user_move = user.clone();
        let identity = context
            .kratos()
            .create_user("email@email.com", "f9456f3c-0398-452a-92c4-15c6f8f3158f")
            .await?;
        context
            .database()
            .write(move |connection| {
                let txn = connection.transaction()?;
                user_move.upsert(&txn)?;
                user_move.create_identity_user(&txn, &Uuid::parse_str(&identity.id).unwrap())?;
                Ok(txn.commit()?)
            })
            .await?;

        let native_login = context
            .kratos()
            .login("email@email.com", "f9456f3c-0398-452a-92c4-15c6f8f3158f")
            .await?;

        // create client
        let mut header_map = HeaderMap::new();
        header_map.insert(
            "X-Session-Token",
            native_login.session_token.unwrap().parse().unwrap(),
        );
        let client = context.client(Some(header_map));

        // test the call
        let retrieved_user = client.get_user().await?.into_inner();
        assert_eq!(retrieved_user.id, user.id);

        Ok(())
    }
}
