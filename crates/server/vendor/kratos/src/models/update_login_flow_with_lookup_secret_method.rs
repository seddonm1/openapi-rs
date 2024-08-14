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

/// UpdateLoginFlowWithLookupSecretMethod : Update Login Flow with Lookup Secret Method
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateLoginFlowWithLookupSecretMethod {
    /// Sending the anti-csrf token is only required for browser login flows.
    #[serde(rename = "csrf_token", skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
    /// The lookup secret.
    #[serde(rename = "lookup_secret")]
    pub lookup_secret: String,
    /// Method should be set to \"lookup_secret\" when logging in using the lookup_secret strategy.
    #[serde(rename = "method")]
    pub method: String,
}

impl UpdateLoginFlowWithLookupSecretMethod {
    /// Update Login Flow with Lookup Secret Method
    pub fn new(lookup_secret: String, method: String) -> UpdateLoginFlowWithLookupSecretMethod {
        UpdateLoginFlowWithLookupSecretMethod {
            csrf_token: None,
            lookup_secret,
            method,
        }
    }
}
