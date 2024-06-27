use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::{AddTask, TaskIden, UpdateTask};

pub struct TaskRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TaskRepository<'a> {
    pub fn create(conn: &'a Connection) -> Self {
        Self { conn }
    }
    pub fn add_task(&self, task: AddTask) -> Result<i64> {
        let sql = Query::insert()
            .into_table(TaskIden::Table)
            .columns([
                TaskIden::Title,
                TaskIden::Info,
                TaskIden::Deadline,
                TaskIden::Status,
                TaskIden::CreatedAt,
            ])
            .values([
                task.title.into(),
                task.info.into(),
                task.deadline.into(),
                task.status.into(),
                task.created_at.into(),
            ])?
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        let id = self.conn.last_insert_rowid();

        Ok(id)
    }
    pub fn edit_task(&self, task: UpdateTask) -> Result<()> {
        let mut sql = Query::update();

        sql.table(TaskIden::Table);

        if let Some(title) = task.title {
            sql.value(TaskIden::Title, title);
        }

        if let Some(info) = task.info {
            sql.value(TaskIden::Info, info);
        }

        if let Some(deadline) = task.deadline {
            sql.value(TaskIden::Deadline, deadline);
        }

        if let Some(status) = task.status {
            sql.value(TaskIden::Status, status);
        }

        if let Some(created_at) = task.created_at {
            sql.value(TaskIden::CreatedAt, created_at);
        }

        sql.and_where(Expr::col(TaskIden::Id).eq(task.id));

        let sql = sql.to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }
}
