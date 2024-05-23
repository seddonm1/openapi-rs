use super::database::Database;

/// Application-specific context
pub struct Context {
    database: Database,
}

impl Context {
    pub fn new(connection: Database) -> Context {
        Context {
            database: connection,
        }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}
