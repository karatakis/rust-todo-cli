use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::{ActionEnum, ActionIden};

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
     * Used to create a single (reversable) action record
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
}
