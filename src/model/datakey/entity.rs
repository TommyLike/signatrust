use crate::model::datakey::traits::{ExtendableAttributes, Identity};
use crate::util::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use secstr::*;

#[derive(Debug, Clone)]
pub enum KeyState {
    Enabled,
    Disabled,
}

impl Default for KeyState {
    fn default() -> Self {
        KeyState::Disabled
    }
}

impl FromStr for KeyState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "enabled" => Ok(KeyState::Enabled),
            "disabled" => Ok(KeyState::Disabled),
            _ => Err(Error::UnsupportedTypeError(format!("unsupported data key state {}", s))),
        }
    }
}

impl Display for KeyState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            KeyState::Enabled => write!(f, "enabled"),
            KeyState::Disabled => write!(f, "disabled"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyType {
    OpenPGP,
    X509
}

impl FromStr for KeyType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "pgp" => Ok(KeyType::OpenPGP),
            "x509" => Ok(KeyType::X509),
            _ => Err(Error::UnsupportedTypeError(format!("unsupported data key type {}", s))),
        }
    }
}

impl Display for KeyType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            KeyType::OpenPGP => write!(f, "pgp"),
            KeyType::X509 => write!(f, "x509"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataKey {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub user: String,
    pub email: String,
    pub attributes: HashMap<String, String>,
    pub key_type: KeyType,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub certificate: Vec<u8>,
    pub create_at: DateTime<Utc>,
    pub expire_at: DateTime<Utc>,
    pub soft_delete: bool,
    pub key_state: KeyState
}

impl ExtendableAttributes for DataKey {
    type Item = HashMap<String, String>;

    fn get_attributes(&self) -> Option<Self::Item> {
        Some(self.attributes.clone())
    }

    fn serialize_attributes(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.attributes)?)
    }
}

impl Identity for DataKey {
    fn get_identity(&self) -> String {
        format!(
            "<ID:{},Name:{}, Email:{},User:{},Type:{}>",
            self.id,
            self.name,
            self.email,
            self.user,
            self.key_type
        )
    }
}
