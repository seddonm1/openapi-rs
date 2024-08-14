use anyhow::Result;
use rusqlite::{params, params_from_iter, Transaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::identity_user::IdentityUser;
use entity_macro::ToSql;

#[cfg(test)]
use crate::test::TestContext;

/// This struct represents a record in the `users` table.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, JsonSchema, Serialize, ToSql)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,
}

impl User {
    /// Creates an  associated `IdentityUser` for this `User`.
    pub fn create_identity_user(&self, txn: &Transaction, id: &Uuid) -> Result<IdentityUser> {
        let identity_user = IdentityUser::new(*id, self.id);
        identity_user.upsert(txn)?;
        Ok(identity_user)
    }

    #[cfg(test)]
    pub async fn test_context() -> Result<TestContext> {
        TestContext::new(vec![]).await
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::test::TestContext;

    pub struct Users;
    impl crate::test::Fixture for Users {
        fn try_fixtures(&self, txn: &Transaction) -> Result<()> {
            serde_json::from_str::<Vec<User>>(&std::fs::read_to_string(std::path::Path::new(
                "fixtures/users.json",
            ))?)?
            .iter()
            .map(|entity| Ok(entity.upsert(txn)?))
            .collect::<Result<Vec<_>>>()?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_upsert_retrieve() -> Result<()> {
        let context = TestContext::new(vec![]).await?;

        let entity = User::default();

        let entity_move = entity.clone();
        context
            .database()
            .write(move |connection| {
                let txn = connection.transaction()?;
                entity_move.upsert(&txn)?;
                Ok(txn.commit()?)
            })
            .await?;

        let entity_move = entity.clone();
        let retrieved_entity = context
            .database()
            .read(move |connection| {
                let txn = connection.transaction()?;
                Ok(User::retrieve(&txn, &entity_move.id)?)
            })
            .await?
            .unwrap();
        assert_eq!(retrieved_entity, entity);

        Ok(())
    }
}
