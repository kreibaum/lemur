use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use diesel::SqliteConnection;

use crate::db;

pub struct DatabaseConnection(pub SqliteConnection);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
    where
        S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(DatabaseConnection(db::establish_connection()))
    }
}
