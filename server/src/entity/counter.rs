use anyhow::Result;
use rusqlite::{OptionalExtension, Transaction};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct Counter {
    pub key: Uuid,
    pub value: u32,
}

#[allow(dead_code)]
impl Counter {
    pub fn new(key: Uuid, value: u32) -> Self {
        Self { key, value }
    }

    pub fn upsert(&self, transaction: &Transaction) -> Result<usize> {
        let mut stmt = transaction.prepare_cached(include_str!("sql/counter/upsert.sql"))?;

        Ok(stmt.execute((&self.key, &self.value))?)
    }

    pub fn retrieve(transaction: &Transaction, key: &Uuid) -> Result<Option<Self>> {
        let mut stmt = transaction.prepare_cached(include_str!("sql/counter/retrieve.sql"))?;

        Ok(stmt
            .query_row([key], |row| {
                Ok(Self {
                    key: key.to_owned(),
                    value: row.get(0)?,
                })
            })
            .optional()?)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{entity::test::Fixture, routes::test::TestContext};
    use csv::ReaderBuilder;
    use serde::Deserialize;
    use std::path::Path;

    #[derive(Clone, Debug, Deserialize)]
    struct CounterFixture {
        key: Uuid,
        value: u32,
    }

    pub struct Counters;
    impl Fixture for Counters {
        fn name(&self) -> &str {
            "Counters"
        }

        /// Load tab-separated data from a file and insert into the database
        fn try_fixtures(&self, transaction: &Transaction) -> Result<()> {
            let fixtures = [Path::new("fixtures/counter.tsv")]
                .into_iter()
                .map(|fixture| {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(b'\t')
                        .has_headers(true)
                        .quoting(false)
                        .from_path(fixture)?;
                    reader
                        .deserialize::<CounterFixture>()
                        .map(|fixture| Ok(fixture?))
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            fixtures
                .into_iter()
                .map(|fixture| {
                    Counter {
                        key: fixture.key,
                        value: fixture.value,
                    }
                    .upsert(transaction)
                    .map_err(|err| anyhow::anyhow!(err))
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(())
        }
    }

    #[tokio::test]
    async fn test_upsert() -> Result<()> {
        let context = TestContext::new(vec![Box::new(Counters)]).await?;

        context
            .database()
            .write(|connection| {
                let transaction = connection.transaction()?;
                let key = Uuid::parse_str("e2268234-9d3d-4ab2-9b68-ec6088f8074b").unwrap();
                let mut counter = Counter::retrieve(&transaction, &key)?.unwrap();
                counter.value = 1;
                assert_eq!(counter.upsert(&transaction)?, 1);
                Ok(transaction.commit()?)
            })
            .await?;

        let counter = context
            .database()
            .read(|connection| {
                let transaction = connection.transaction()?;
                let key = Uuid::parse_str("e2268234-9d3d-4ab2-9b68-ec6088f8074b").unwrap();
                Ok(Counter::retrieve(&transaction, &key)?)
            })
            .await?;

        assert_eq!(counter.unwrap().value, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve() -> Result<()> {
        let context = TestContext::new(vec![Box::new(Counters)]).await?;

        let counter = context
            .database()
            .read(|connection| {
                let transaction = connection.transaction()?;
                let key = Uuid::parse_str("e2268234-9d3d-4ab2-9b68-ec6088f8074b").unwrap();

                Ok(Counter::retrieve(&transaction, &key)?)
            })
            .await?;

        assert_eq!(counter.unwrap().value, 10);

        Ok(())
    }
}
