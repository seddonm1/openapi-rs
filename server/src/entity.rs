pub mod counter;

#[cfg(test)]
pub mod test {
    use anyhow::Result;
    use rusqlite::Transaction;

    /// Fixture allows tests to configure database state before a test begins.
    pub trait Fixture: Send + Sync {
        fn name(&self) -> &str;
        fn try_fixtures(&self, transaction: &Transaction<'_>) -> Result<()>;
    }
}
