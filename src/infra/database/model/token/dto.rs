use crate::util::error::Result;



use sqlx::FromRow;
use chrono::{DateTime, Utc};





use crate::domain::token::entity::Token;

#[derive(Debug, FromRow)]
pub(super) struct TokenDTO {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expire_at: DateTime<Utc>,
}

impl TokenDTO {
    pub async fn encrypt(
        token: &Token) -> Result<Self> {
        Ok(Self {
            id: token.id,
            user_id: token.user_id,
            token: token.token.clone(),
            expire_at: token.expire_at,
        })
    }
    pub async fn decrypt(&self) -> Result<Token> {
        Ok(Token {
            id: self.id,
            user_id: self.user_id,
            token: self.token.clone(),
            expire_at:self.expire_at,
        })
    }
}
