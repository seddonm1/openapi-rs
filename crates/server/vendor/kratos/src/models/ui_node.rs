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

/// UiNode : Nodes are represented as HTML elements or their native UI equivalents. For example, a node can be an `<img>` tag, or an `<input element>` but also `some plain text`.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiNode {
    #[serde(rename = "attributes")]
    pub attributes: Box<models::UiNodeAttributes>,
    /// Group specifies which group (e.g. password authenticator) this node belongs to. default DefaultGroup password PasswordGroup oidc OpenIDConnectGroup profile ProfileGroup link LinkGroup code CodeGroup totp TOTPGroup lookup_secret LookupGroup webauthn WebAuthnGroup passkey PasskeyGroup
    #[serde(rename = "group")]
    pub group: Group,
    #[serde(rename = "messages")]
    pub messages: Vec<models::UiText>,
    #[serde(rename = "meta")]
    pub meta: Box<models::UiNodeMeta>,
    /// The node's type text Text input Input img Image a Anchor script Script
    #[serde(rename = "type")]
    pub r#type: Type,
}

impl UiNode {
    /// Nodes are represented as HTML elements or their native UI equivalents. For example, a node can be an `<img>` tag, or an `<input element>` but also `some plain text`.
    pub fn new(attributes: models::UiNodeAttributes, group: Group, messages: Vec<models::UiText>, meta: models::UiNodeMeta, r#type: Type) -> UiNode {
        UiNode {
            attributes: Box::new(attributes),
            group,
            messages,
            meta: Box::new(meta),
            r#type,
        }
    }
}
/// Group specifies which group (e.g. password authenticator) this node belongs to. default DefaultGroup password PasswordGroup oidc OpenIDConnectGroup profile ProfileGroup link LinkGroup code CodeGroup totp TOTPGroup lookup_secret LookupGroup webauthn WebAuthnGroup passkey PasskeyGroup
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Group {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "oidc")]
    Oidc,
    #[serde(rename = "profile")]
    Profile,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "totp")]
    Totp,
    #[serde(rename = "lookup_secret")]
    LookupSecret,
    #[serde(rename = "webauthn")]
    Webauthn,
    #[serde(rename = "passkey")]
    Passkey,
}

impl Default for Group {
    fn default() -> Group {
        Self::Default
    }
}
/// The node's type text Text input Input img Image a Anchor script Script
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "img")]
    Img,
    #[serde(rename = "a")]
    A,
    #[serde(rename = "script")]
    Script,
}

impl Default for Type {
    fn default() -> Type {
        Self::Text
    }
}
