use anyhow::Result;
use entity_macro::ToSql;
use rusqlite::{params, params_from_iter, Transaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::User;

#[cfg(test)]
use crate::test::TestContext;

/// This struct represents a record in the `identitys_users` table.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, JsonSchema, Serialize, ToSql)]
pub struct IdentityUser {
    /// The unique identifier for the identity user.
    pub id: Uuid,
    /// The unique identifier for the associated user.
    pub user_id: Uuid,
}

impl IdentityUser {
    /// Retrieves the associated `User` for this `IdentityUser`.
    pub fn retrieve_user(&self, txn: &Transaction) -> Result<User> {
        Ok(User::retrieve(txn, &self.user_id)?.unwrap())
    }
}

#[cfg(test)]
pub mod test {
    use crate::entity::user::test::Users;

    use super::*;

    pub struct IdentitysUsers;
    impl crate::test::Fixture for IdentitysUsers {
        fn try_fixtures(&self, txn: &Transaction) -> Result<()> {
            serde_json::from_str::<Vec<IdentityUser>>(&std::fs::read_to_string(
                std::path::Path::new("fixtures/identitys_users.json"),
            )?)?
            .iter()
            .map(|entity| Ok(entity.upsert(txn)?))
            .collect::<Result<Vec<_>>>()?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_upsert_retrieve() -> Result<()> {
        let context = TestContext::new(vec![Box::new(Users)]).await?;

        let entity = IdentityUser::new(
            Uuid::parse_str("6950e471-e464-7a13-c4f5-b565cba03720").unwrap(),
            Uuid::parse_str("3d517fe6-ebab-7b8c-fcf9-8db6259c8a59").unwrap(),
        );

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
                Ok(IdentityUser::retrieve(&txn, &entity_move.id)?)
            })
            .await?
            .unwrap();

        assert_eq!(retrieved_entity, entity);

        Ok(())
    }
}
