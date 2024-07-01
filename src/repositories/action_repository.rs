use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::{Action, ActionEnum, ActionIden};

/**
 * Action database repository
 */
pub struct ActionRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ActionRepository<'a> {
    /**
     * Used to initialize the repository
     */
    pub fn create(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /**
     * Used to create a single (reversible) action record
     */
    pub fn create_action(&self, action: ActionEnum, now: &str) -> Result<i64> {
        // create new action
        let sql = Query::insert()
            .into_table(ActionIden::Table)
            .columns([
                ActionIden::Action,
                ActionIden::Restored,
                ActionIden::CreatedAt,
            ])
            .values([action.to_blob().into(), false.into(), now.into()])?
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

        let id = self.conn.last_insert_rowid();

        // delete restored actions
        let sql = Query::delete()
            .from_table(ActionIden::Table)
            .and_where(Expr::col(ActionIden::Restored).eq(true))
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

        Ok(id)
    }

    /**
     * Used to create a single (reversible) action record
     */
    pub fn update_action(&self, id: i64, action: ActionEnum, restored: bool) -> Result<()> {
        let sql = Query::update()
            .table(ActionIden::Table)
            .values([
                (ActionIden::Action, action.to_blob().into()),
                (ActionIden::Restored, restored.into()),
            ])
            .and_where(Expr::col(ActionIden::Id).eq(id))
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to get the last non restored Action log record
     */
    pub fn get_last_unrestored_action(&self) -> Result<Action> {
        let sql = Query::select()
            .from(ActionIden::Table)
            .columns([
                ActionIden::Id,
                ActionIden::Action,
                ActionIden::Restored,
                ActionIden::CreatedAt,
            ])
            .and_where(Expr::col(ActionIden::Restored).eq(false))
            .order_by(ActionIden::Id, sea_query::Order::Desc)
            .limit(1)
            .to_string(SqliteQueryBuilder);

        let action: Action = self.conn.query_row(&sql, (), |row| {
            Ok(Action {
                id: row.get(0)?,
                action: row.get(1)?,
                restored: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        Ok(action)
    }

    /**
     * Used to get the last restored Action log record
     */
    pub fn get_first_restored_action(&self) -> Result<Action> {
        let sql = Query::select()
            .from(ActionIden::Table)
            .columns([
                ActionIden::Id,
                ActionIden::Action,
                ActionIden::Restored,
                ActionIden::CreatedAt,
            ])
            .and_where(Expr::col(ActionIden::Restored).eq(true))
            .order_by(ActionIden::Id, sea_query::Order::Asc)
            .limit(1)
            .to_string(SqliteQueryBuilder);

        let action: Action = self.conn.query_row(&sql, (), |row| {
            Ok(Action {
                id: row.get(0)?,
                action: row.get(1)?,
                restored: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        Ok(action)
    }

    /** Used to fetch all actions */
    pub fn fetch_actions(&self, limit: u64) -> Result<Vec<Action>> {
        let sql = Query::select()
            .from(ActionIden::Table)
            .columns([
                ActionIden::Id,
                ActionIden::Action,
                ActionIden::Restored,
                ActionIden::CreatedAt,
            ])
            .order_by(ActionIden::Id, sea_query::Order::Desc)
            .limit(limit)
            .to_string(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql)?;

        let rows = stmt
            .query_map((), |row| {
                Ok(Action {
                    id: row.get(0)?,
                    action: row.get(1)?,
                    restored: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<Action>, _>>()?;

        Ok(rows)
    }

    /**
     * Used to delete all actions
     */
    pub fn delete_all(&self) -> Result<i64> {
        let sql = Query::select()
            .from(ActionIden::Table)
            .expr(Expr::col(ActionIden::Id).count())
            .to_string(SqliteQueryBuilder);

        let count = self.conn.query_row(&sql, (), |row| Ok(row.get(0)?))?;

        let sql = Query::delete()
            .from_table(ActionIden::Table)
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rusqlite::Connection;

    use crate::{
        models::{setup_database, Action, ActionEnum},
        repositories::get_now,
    };

    use super::ActionRepository;

    #[test]
    fn test_create_update() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let repository = ActionRepository::create(&conn);

        let now = get_now();

        let action = ActionEnum::BatchCategoryRename {
            old_category: "test".into(),
            new_category: "tost".into(),
        };

        // test create
        let id = {
            let id = repository.create_action(action.clone(), &now.to_string())?;
            let actions = repository.fetch_actions(1)?;
            assert_eq!(
                vec![Action {
                    id,
                    created_at: now,
                    action,
                    restored: false
                }],
                actions
            );
            id
        };

        // test update
        {
            let action = ActionEnum::BatchCategoryRename {
                old_category: "1".into(),
                new_category: "2".into(),
            };
            repository.update_action(id, action.clone(), true)?;
            let actions = repository.fetch_actions(1)?;
            assert_eq!(
                vec![Action {
                    id,
                    created_at: now,
                    action,
                    restored: true
                }],
                actions
            );
        }

        Ok(())
    }

    #[test]
    fn test_get_last_first_unrestored_restored_action() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let repository = ActionRepository::create(&conn);

        let now = get_now();

        let old_texts = vec!["one", "two", "three", "four"];
        let new_texts = vec!["1", "2", "3", "4"];

        for id in 0..4 {
            let action = ActionEnum::BatchCategoryRename {
                old_category: old_texts.get(id).unwrap().to_string(),
                new_category: new_texts.get(id).unwrap().to_string(),
            };

            repository.create_action(action, &now.to_string())?;
        }

        for id in 2..4 {
            let action = ActionEnum::BatchCategoryRename {
                old_category: old_texts.get(id).unwrap().to_string(),
                new_category: new_texts.get(id).unwrap().to_string(),
            };

            repository.update_action((id + 1) as i64, action, true)?;
        }

        // test get last unrestored
        {
            let last = repository.get_last_unrestored_action()?;
            assert_eq!(
                Action {
                    id: 2,
                    created_at: now,
                    action: ActionEnum::BatchCategoryRename {
                        old_category: old_texts.get(1).unwrap().to_string(),
                        new_category: new_texts.get(1).unwrap().to_string(),
                    },
                    restored: false
                },
                last
            );
        }

        // test get first restored
        {
            let first = repository.get_first_restored_action()?;
            assert_eq!(
                Action {
                    id: 3,
                    created_at: now,
                    action: ActionEnum::BatchCategoryRename {
                        old_category: old_texts.get(2).unwrap().to_string(),
                        new_category: new_texts.get(2).unwrap().to_string(),
                    },
                    restored: true
                },
                first
            );
        }

        Ok(())
    }
}
