use std::collections::HashMap;

use crate::database::DbSchema;

#[derive(Clone, Debug)]
pub struct AppData {
    pub database_schemas: HashMap<String, DbSchema>,
}
