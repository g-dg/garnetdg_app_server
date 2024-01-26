//! Authentication module

use crate::{config::AuthenticationConfig, database::DbSchema};

#[derive(Clone)]
pub struct Auth {
    config: AuthenticationConfig,
    db_schema: DbSchema,
}

impl Auth {
    pub fn new(config: &AuthenticationConfig, db_schema: DbSchema) -> Self {
        Self {
            config: config.clone(),
            db_schema,
        }
    }
}
