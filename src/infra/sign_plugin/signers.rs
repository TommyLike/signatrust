use crate::domain::sign_plugin::SignPlugins;
use crate::infra::sign_plugin::openpgp::OpenPGPPlugin;
use crate::infra::sign_plugin::x509::X509Plugin;
use crate::domain::datakey::entity::{KeyType};
use crate::util::error::Result;
use std::collections::HashMap;

use crate::domain::datakey::entity::SecDataKey;

pub struct Signers {}

impl Signers {

    //get responding sign plugin for data signing
    pub fn load_from_data_key(key_type: &KeyType, data_key: &SecDataKey) -> Result<Box<dyn SignPlugins>> {
        match key_type {
            KeyType::OpenPGP => Ok(Box::new(OpenPGPPlugin::new(data_key)?)),
            KeyType::X509 => Ok(Box::new(X509Plugin::new(data_key)?)),
        }
    }

    //generating new key, including private & public keys and the certificate, empty if not required.
    pub fn generate_keys(
        key_type: &KeyType,
        value: &HashMap<String, String>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        match key_type {
            KeyType::OpenPGP => OpenPGPPlugin::generate_keys(value),
            KeyType::X509 => X509Plugin::generate_keys(value),
        }
    }
}
