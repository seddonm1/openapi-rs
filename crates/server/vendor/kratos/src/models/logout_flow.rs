/*
 * Ory Identities API
 *
 * This is the API specification for Ory Identities with features such as registration, login, recovery, account verification, profile settings, password reset, identity management, session management, email and sms delivery, and more. 
 *
 * The version of the OpenAPI document: 
 * Contact: office@ory.sh
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// LogoutFlow : Logout Flow
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogoutFlow {
    /// LogoutToken can be used to perform logout using AJAX.
    #[serde(rename = "logout_token")]
    pub logout_token: String,
    /// LogoutURL can be opened in a browser to sign the user out.  format: uri
    #[serde(rename = "logout_url")]
    pub logout_url: String,
}

impl LogoutFlow {
    /// Logout Flow
    pub fn new(logout_token: String, logout_url: String) -> LogoutFlow {
        LogoutFlow {
            logout_token,
            logout_url,
        }
    }
}
