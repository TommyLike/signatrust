use super::traits::SignPlugins;
use crate::model::datakey::entity::DataKey;
use crate::model::datakey::traits::Identity;
use crate::util::error::{Error, Result};
use chrono::Utc;
use pgp::composed::signed_key::SignedSecretKey;
use pgp::composed::{key::SecretKeyParamsBuilder, KeyDetails, KeyType, SecretKey, SecretSubkey};
use pgp::crypto::{hash::HashAlgorithm, sym::SymmetricKeyAlgorithm};
use pgp::packet::SignatureConfig;
use pgp::packet::*;
use pgp::packet::{KeyFlags, UserAttribute, UserId};
use pgp::types::KeyTrait;
use pgp::types::{CompressionAlgorithm, PublicKeyTrait, SecretKeyTrait};
use pgp::Deserializable;
use serde::Deserialize;
use smallvec::*;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::str::from_utf8;
use std::str::FromStr;
use validator::{Validate, ValidationError};
use pgp::composed::StandaloneSignature;

const DETACHED_SIGNATURE: &str = "detached";

#[derive(Debug, Validate, Deserialize)]
pub struct KeyGenerationParameter {
    #[validate(length(min = 4, max = 20))]
    name: String,
    #[validate(email)]
    email: String,
    #[validate(custom = "validate_key_type")]
    key_type: String,
    #[validate(custom = "validate_key_size")]
    key_length: String,
}

impl KeyGenerationParameter {
    pub fn get_key(&self) -> Result<KeyType> {
        return match self.key_type.as_str() {
            "rsa" => Ok(KeyType::Rsa(self.key_length.parse::<u32>().unwrap())),
            "ecdh" => Ok(KeyType::ECDH),
            "eddsa" => Ok(KeyType::EdDSA),
            _ => Err(Error::ParameterError(
                "invalid key type for openpgp".to_string(),
            )),
        };
    }

    pub fn get_user_id(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}

fn validate_key_type(key_type: &str) -> std::result::Result<(), ValidationError> {
    if !vec!["rsa", "ecdh", "eddsa"].contains(&key_type) {
        return Err(ValidationError::new("invalid key type"));
    }
    Ok(())
}

fn validate_key_size(key_size: &str) -> std::result::Result<(), ValidationError> {
    if !vec!["1024", "2048", "3072", "4096"].contains(&key_size) {
        return Err(ValidationError::new("invalid key size"));
    }
    Ok(())
}

pub struct OpenPGPPlugin {
    secret_key: SignedSecretKey,
    identity: String,
}

impl OpenPGPPlugin {
    pub fn attributes_validate(attr: HashMap<String, String>) -> Result<KeyGenerationParameter> {
        let parameter: KeyGenerationParameter =
            serde_json::from_str(serde_json::to_string(&attr)?.as_str())?;
        match parameter.validate() {
            Ok(_) => Ok(parameter),
            Err(e) => Err(Error::ParameterError(format!("{:?}", e))),
        }
    }
}

impl SignPlugins for OpenPGPPlugin {
    fn new(db: DataKey) -> Result<Self> {
        let value = from_utf8(&db.private_key).map_err(|e| Error::KeyParseError(e.to_string()))?;
        let (secret_key, _) =
            SignedSecretKey::from_string(value).map_err(|e| Error::KeyParseError(e.to_string()))?;
        Ok(Self {
            secret_key,
            identity: db.get_identity(),
        })
    }

    fn parse_attributes(
        private_key: Option<Vec<u8>>,
        public_key: Option<Vec<u8>>,
        certificate: Option<Vec<u8>>,
    ) -> HashMap<String, String> {
        todo!()
    }

    fn generate_keys(
        value: HashMap<String, String>,
    ) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>)> {
        let parameter = OpenPGPPlugin::attributes_validate(value)?;
        let mut key_params = SecretKeyParamsBuilder::default();
        key_params
            .key_type(parameter.get_key()?)
            .can_create_certificates(false)
            .can_sign(true)
            .primary_user_id(parameter.get_user_id())
            .preferred_symmetric_algorithms(smallvec![SymmetricKeyAlgorithm::AES256,])
            .preferred_hash_algorithms(smallvec![HashAlgorithm::SHA2_256,])
            .preferred_compression_algorithms(smallvec![CompressionAlgorithm::ZLIB,]);
        let secret_key_params = key_params.build()?;
        let secret_key = secret_key_params.generate()?;
        let passwd_fn = || String::new();
        let signed_secret_key = secret_key.sign(passwd_fn)?;
        let public_key = signed_secret_key.public_key();
        let signed_public_key = public_key.sign(&signed_secret_key, passwd_fn)?;
        Ok((
            Some(signed_secret_key.to_armored_bytes(None)?),
            Some(signed_public_key.to_armored_bytes(None)?),
            None,
        ))
    }

    fn sign(&self, content: Vec<u8>, options: HashMap<String, String>) -> Result<Vec<u8>> {
        let passwd_fn = String::new;
        let now = Utc::now();
        let sig_cfg = SignatureConfig {
            version: SignatureVersion::V4,
            typ: SignatureType::Binary,
            //todo update the pub alg and hash alg to the corresponding algorithm to data key
            pub_alg: ::pgp::crypto::public_key::PublicKeyAlgorithm::RSA,
            hash_alg: HashAlgorithm::SHA2_256,
            issuer: Some(self.secret_key.key_id()),
            created: Some(now),
            unhashed_subpackets: vec![],
            hashed_subpackets: vec![
                Subpacket::SignatureCreationTime(now),
                Subpacket::Issuer(self.secret_key.key_id()),
            ],
        };
        let read_cursor = Cursor::new(content);
        let signature_packet = sig_cfg
            .sign(&self.secret_key, passwd_fn, read_cursor)
            .map_err(|e| Error::SignError(self.identity.clone(), e.to_string()))?;


        //detached signature
        if let Some(detached) = options.get(DETACHED_SIGNATURE) {
            if detached == "true" {
                let standard_signature = StandaloneSignature::new(signature_packet);
                return Ok(standard_signature.to_armored_bytes(None)?)
            }
        }
        let mut signature_bytes = Vec::with_capacity(1024);
        let mut cursor = Cursor::new(&mut signature_bytes);
        write_packet(&mut cursor, &signature_packet)
            .map_err(|e| Error::SignError(self.identity.clone(), e.to_string()))?;
        Ok(signature_bytes)
    }
}