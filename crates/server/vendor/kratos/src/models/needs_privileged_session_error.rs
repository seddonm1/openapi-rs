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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeedsPrivilegedSessionError {
    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
    pub error: Option<Box<models::GenericError>>,
    /// Points to where to redirect the user to next.
    #[serde(rename = "redirect_browser_to")]
    pub redirect_browser_to: String,
}

impl NeedsPrivilegedSessionError {
    pub fn new(redirect_browser_to: String) -> NeedsPrivilegedSessionError {
        NeedsPrivilegedSessionError {
            error: None,
            redirect_browser_to,
        }
    }
}

