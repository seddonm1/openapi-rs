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

/// ContinueWithVerificationUi : Indicates, that the UI flow could be continued by showing a verification ui
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContinueWithVerificationUi {
    #[serde(rename = "flow")]
    pub flow: Box<models::ContinueWithVerificationUiFlow>,
}

impl ContinueWithVerificationUi {
    /// Indicates, that the UI flow could be continued by showing a verification ui
    pub fn new(flow: models::ContinueWithVerificationUiFlow) -> ContinueWithVerificationUi {
        ContinueWithVerificationUi {
            flow: Box::new(flow),
        }
    }
}
/// Action will always be `show_verification_ui` show_verification_ui ContinueWithActionShowVerificationUIString
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "show_verification_ui")]
    ShowVerificationUi,
}

impl Default for Action {
    fn default() -> Action {
        Self::ShowVerificationUi
    }
}
