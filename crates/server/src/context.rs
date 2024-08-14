use crate::kratos::Kratos;

use super::database::Database;

/// Application-specific context
pub struct Context {
    database: Database,
    kratos: Kratos,
}

impl Context {
    pub fn new(database: Database, kratos: Kratos) -> Context {
        Context { database, kratos }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn kratos(&self) -> &Kratos {
        &self.kratos
    }
}
