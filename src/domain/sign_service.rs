use crate::util::error::{Result, Error};
use std::collections::HashMap;
use std::str::FromStr;

use crate::domain::datakey::entity::DataKey;
use async_trait::async_trait;

#[derive(Debug)]
pub enum SignBackendType {
    Memory,
}

impl FromStr for SignBackendType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "memory" => Ok(SignBackendType::Memory),
            _ => Err(Error::UnsupportedTypeError(format!("{} sign backend type", s))),
        }
    }
}

#[async_trait]
pub trait SignBackend: Send + Sync{
    async fn generate_keys(&self, data_key: &mut DataKey) -> Result<()>;
    async fn sign(&self, data_key: &DataKey, content: Vec<u8>, options: HashMap<String, String>) -> Result<Vec<u8>>;
    async fn decode_public_keys(&self, data_key: &mut DataKey) -> Result<()>;
}
