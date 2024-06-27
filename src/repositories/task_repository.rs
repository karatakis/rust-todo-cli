use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::{AddTask, Task, TaskIden, UpdateTask};

pub struct TaskRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TaskRepository<'a> {
    pub fn create(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_task(&self, id: i64) -> Result<Option<Task>> {
        let sql = Query::select()
            .from(TaskIden::Table)
            .columns([
                TaskIden::Id,
                TaskIden::Title,
                TaskIden::Info,
                TaskIden::Deadline,
                TaskIden::Status,
                TaskIden::CreatedAt,
            ])
            .and_where(Expr::col(TaskIden::Id).eq(id))
            .to_string(SqliteQueryBuilder);

        let result = self.conn.query_row(&sql, (), |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                info: row.get(2)?,
                deadline: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        });

        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn add_task(&self, task: AddTask) -> Result<Task> {
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
                task.title.clone().into(),
                task.info.clone().into(),
                task.deadline.into(),
                task.status.into(),
                task.created_at.into(),
            ])?
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        let id = self.conn.last_insert_rowid();

        Ok(Task {
            id,
            title: task.title,
            info: task.info,
            deadline: task.deadline,
            status: task.status,
            created_at: task.created_at,
        })
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

    pub fn delete_task(&self, id: i64) -> Result<()> {
        let sql = Query::delete()
            .from_table(TaskIden::Table)
            .and_where(Expr::col(TaskIden::Id).eq(id))
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

        Ok(())
    }
}
