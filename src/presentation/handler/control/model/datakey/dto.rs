use crate::domain::datakey::entity::{DataKey, KeyState};
use crate::domain::datakey::entity::KeyType;

use crate::util::error::Result;

use chrono::{DateTime, Utc};





use std::str::FromStr;


use validator::{Validate, ValidationError};
use std::collections::HashMap;
use crate::util::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ExportKey {
    pub public_key: String,
    pub certificate: String,
}

impl TryFrom<DataKey> for ExportKey {
    type Error = Error;

    fn try_from(value: DataKey) -> std::result::Result<Self, Self::Error> {
        Ok(ExportKey{
            public_key: String::from_utf8_lossy(&value.public_key).to_string(),
            certificate: String::from_utf8_lossy(&value.certificate).to_string()
        })
    }
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct DataKeyDTO {
    #[serde(skip_deserializing)]
    pub id: i32,
    #[validate(length(min = 4, max = 20))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 0, max = 100))]
    pub description: String,
    pub user: String,
    pub attributes: HashMap<String, String>,
    pub key_type: String,
    #[validate(custom = "validate_utc_time")]
    pub create_at: String,
    #[validate(custom = "validate_utc_time")]
    pub expire_at: String,
    #[serde(skip_deserializing)]
    pub key_state: String,
}

fn validate_utc_time(expire: &str) -> std::result::Result<(), ValidationError> {
    if expire.parse::<DateTime<Utc>>().is_err() {
        return Err(ValidationError::new("failed to parse time string to utc"));
    }
    Ok(())
}

impl TryFrom<DataKeyDTO> for DataKey {
    type Error = Error;

    fn try_from(dto: DataKeyDTO) -> Result<Self> {
        let mut combined_attributes = dto.attributes.clone();
        combined_attributes.insert("name".to_string(), dto.name.clone());
        combined_attributes.insert("email".to_string(), dto.email.clone());
        combined_attributes.insert("create_at".to_string(), dto.create_at.clone());
        combined_attributes.insert("expire_at".to_string(), dto.expire_at.clone());
        Ok(DataKey {
            id: dto.id,
            name: dto.name,
            description: dto.description,
            user: dto.user,
            email: dto.email,
            attributes: combined_attributes,
            key_type: KeyType::from_str(dto.key_type.as_str())?,
            private_key: vec![],
            public_key: vec![],
            certificate: vec![],
            create_at: dto.create_at.parse()?,
            expire_at: dto.expire_at.parse()?,
            soft_delete: false,
            key_state: KeyState::default()
        })
    }
}

impl TryFrom<DataKey> for DataKeyDTO {
    type Error = Error;

    fn try_from(dto: DataKey) -> Result<Self> {
        Ok(DataKeyDTO {
            id: dto.id,
            name: dto.name,
            description: dto.description,
            user: dto.user,
            email: dto.email,
            attributes: dto.attributes,
            key_type: dto.key_type.to_string(),
            create_at: dto.create_at.to_string(),
            expire_at: dto.expire_at.to_string(),
            key_state: dto.key_state.to_string(),
        })
    }
}
