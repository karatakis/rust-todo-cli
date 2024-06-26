use anyhow::Result;
use rusqlite::Connection;

use crate::models::AddTask;

pub struct TaskRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TaskRepository<'a> {
    pub fn create(conn: &'a Connection) -> Self {
        Self { conn }
    }
    pub fn add_task(&self, task: AddTask) -> Result<i64> {
        self.conn.execute(
            "
                INSERT INTO tasks (
                    title,
                    info,
                    deadline,
                    status,
                    created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5);
        ",
            (
                &task.title,
                &task.info,
                &task.deadline,
                &task.status,
                &task.created_at,
            ),
        )?;

        let id = self.conn.last_insert_rowid();

        Ok(id)
    }
}
