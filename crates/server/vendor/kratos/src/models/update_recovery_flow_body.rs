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

/// UpdateRecoveryFlowBody : Update Recovery Flow Request Body
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum UpdateRecoveryFlowBody {
    #[serde(rename="link")]
    Link(Box<models::UpdateRecoveryFlowWithLinkMethod>),
    #[serde(rename="code")]
    Code(Box<models::UpdateRecoveryFlowWithCodeMethod>),
}

impl Default for UpdateRecoveryFlowBody {
    fn default() -> Self {
        Self::Link(Default::default())
    }
}


