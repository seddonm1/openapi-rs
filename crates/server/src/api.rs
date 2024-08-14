use dropshot::{HttpError, HttpResponseOk, RequestContext};

use crate::entity::user::User;

/// The Dropshot API trait.
#[dropshot::api_description]
pub(crate) trait ServerApi {
    /// By default, the name of the context type is Context. To specify a
    /// different name, use the { context = ... } attribute on
    /// `#[dropshot::api_description]`.
    type Context;

    /// Get the value of the counter.
    #[endpoint { method = GET, path = "/v1/user" }]
    async fn get_user(
        rqctx: RequestContext<Self::Context>,
    ) -> Result<HttpResponseOk<User>, HttpError>;
}

// A simple function to generate an OpenAPI spec for the trait, without having
// a real implementation available.
//
// If the interface and implementation (see below) are in different crates, then
// this function would live in the interface crate.
pub(crate) fn generate_openapi_spec() -> String {
    let description = server_api_mod::stub_api_description().unwrap();
    let spec = description.openapi("Server", "1.0.0");
    serde_json::to_string_pretty(&spec.json().unwrap()).unwrap()
}
