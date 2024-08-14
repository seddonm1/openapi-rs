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

/// UpdateSettingsFlowWithWebAuthnMethod : Update Settings Flow with WebAuthn Method
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateSettingsFlowWithWebAuthnMethod {
    /// CSRFToken is the anti-CSRF token
    #[serde(rename = "csrf_token", skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
    /// Method  Should be set to \"webauthn\" when trying to add, update, or remove a webAuthn pairing.
    #[serde(rename = "method")]
    pub method: String,
    /// Transient data to pass along to any webhooks
    #[serde(rename = "transient_payload", skip_serializing_if = "Option::is_none")]
    pub transient_payload: Option<serde_json::Value>,
    /// Register a WebAuthn Security Key  It is expected that the JSON returned by the WebAuthn registration process is included here.
    #[serde(rename = "webauthn_register", skip_serializing_if = "Option::is_none")]
    pub webauthn_register: Option<String>,
    /// Name of the WebAuthn Security Key to be Added  A human-readable name for the security key which will be added.
    #[serde(rename = "webauthn_register_displayname", skip_serializing_if = "Option::is_none")]
    pub webauthn_register_displayname: Option<String>,
    /// Remove a WebAuthn Security Key  This must contain the ID of the WebAuthN connection.
    #[serde(rename = "webauthn_remove", skip_serializing_if = "Option::is_none")]
    pub webauthn_remove: Option<String>,
}

impl UpdateSettingsFlowWithWebAuthnMethod {
    /// Update Settings Flow with WebAuthn Method
    pub fn new(method: String) -> UpdateSettingsFlowWithWebAuthnMethod {
        UpdateSettingsFlowWithWebAuthnMethod {
            csrf_token: None,
            method,
            transient_payload: None,
            webauthn_register: None,
            webauthn_register_displayname: None,
            webauthn_remove: None,
        }
    }
}

