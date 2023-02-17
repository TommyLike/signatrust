use crate::infra::kms::kms_provider::KMSProvider;
use crate::model::clusterkey::entity::ClusterKey;
use crate::util::error::Result;
use crate::util::key;
use hex;
use sqlx::types::chrono;
use sqlx::FromRow;
use std::boxed::Box;
use std::convert::identity;
use std::ops::Deref;
use std::sync::Arc;
use secstr::SecVec;

#[derive(Debug, FromRow)]
pub(super) struct ClusterKeyDTO {
    pub id: i32,
    pub data: Vec<u8>,
    pub algorithm: String,
    pub identity: String,
    pub create_at: chrono::DateTime<chrono::Utc>,
    pub expire_at: chrono::DateTime<chrono::Utc>,
}

impl ClusterKeyDTO {
    pub async fn encrypt(
        cluster_key: &ClusterKey,
        kms_provider: &Arc<Box<dyn KMSProvider>>,
    ) -> Result<Self> {
        Ok(Self {
            id: cluster_key.id,
            data: kms_provider
                .encode(key::encode_u8_to_hex_string(&cluster_key.data.unsecure()))
                .await?
                .as_bytes()
                .to_vec(),
            algorithm: cluster_key.algorithm.clone(),
            identity: cluster_key.identity.clone(),
            create_at: cluster_key.create_at,
            expire_at: cluster_key.expire_at,
        })
    }
    pub async fn decrypt(&self, kms_provider: &Arc<Box<dyn KMSProvider>>) -> Result<ClusterKey> {
        Ok(ClusterKey {
            id: self.id,
            data: SecVec::new(key::decode_hex_string_to_u8(
                &kms_provider
                    .decode(String::from_utf8(self.data.clone())?)
                    .await?,
            )),
            algorithm: self.algorithm.clone(),
            identity: self.identity.clone(),
            create_at: self.create_at,
            expire_at: self.expire_at,
        })
    }
}
