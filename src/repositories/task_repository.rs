use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::{AddTask, Task, TaskIden, UpdateTask};

/**
 * Task database repository
 */
pub struct TaskRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TaskRepository<'a> {
    /**
     * Used to initialize the repository
     */
    pub fn create(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /**
     * Used to fetch a single task
     */
    pub fn get_task(&self, id: i64) -> Result<Option<Task>> {
        let sql = Query::select()
            .from(TaskIden::Table)
            .columns([
                TaskIden::Id,
                TaskIden::Title,
                TaskIden::Info,
                TaskIden::Deadline,
                TaskIden::Status,
                TaskIden::UpdatedAt,
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
                updated_at: row.get(5)?,
                created_at: row.get(6)?,
                categories: None,
            })
        });

        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /**
     * Used to create a single task
     */
    pub fn create_task(&self, task: AddTask) -> Result<Task> {
        let sql = Query::insert()
            .into_table(TaskIden::Table)
            .columns([
                TaskIden::Title,
                TaskIden::Info,
                TaskIden::Deadline,
                TaskIden::Status,
                TaskIden::UpdatedAt,
                TaskIden::CreatedAt,
            ])
            .values([
                task.title.clone().into(),
                task.info.clone().into(),
                task.deadline.into(),
                task.status.into(),
                task.created_at.into(),
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
            updated_at: task.created_at,
            created_at: task.created_at,
            categories: None,
        })
    }

    /**
     * Used to create a single task with specified id
     */
    pub fn create_task_with_id(&self, id: i64, task: AddTask) -> Result<Task> {
        let sql = Query::insert()
            .into_table(TaskIden::Table)
            .columns([
                TaskIden::Id,
                TaskIden::Title,
                TaskIden::Info,
                TaskIden::Deadline,
                TaskIden::Status,
                TaskIden::UpdatedAt,
                TaskIden::CreatedAt,
            ])
            .values([
                id.into(),
                task.title.clone().into(),
                task.info.clone().into(),
                task.deadline.into(),
                task.status.into(),
                task.created_at.into(),
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
            updated_at: task.created_at,
            created_at: task.created_at,
            categories: None,
        })
    }

    /**
     * Used to update a single task
     */
    pub fn update_task(&self, id: i64, new_task: UpdateTask, now: &str) -> Result<()> {
        let mut changes = 0;

        let mut sql = Query::update();

        sql.table(TaskIden::Table);

        // update only the changed fields
        if let Some(title) = new_task.title {
            sql.value(TaskIden::Title, title);
            changes += 1;
        }

        if let Some(info) = new_task.info {
            sql.value(TaskIden::Info, info);
            changes += 1;
        }

        if let Some(deadline) = new_task.deadline {
            sql.value(TaskIden::Deadline, deadline);
            changes += 1;
        }

        if let Some(status) = new_task.status {
            sql.value(TaskIden::Status, status);
            changes += 1;
        }

        if let Some(created_at) = new_task.created_at {
            sql.value(TaskIden::CreatedAt, created_at);
            changes += 1;
        }

        // if no changes then return error
        if changes > 0 {
            sql.value(TaskIden::UpdatedAt, now);
            sql.and_where(Expr::col(TaskIden::Id).eq(id));
            let sql = sql.to_string(SqliteQueryBuilder);
            self.conn.execute(&sql, ())?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("No changes found"))
        }
    }

    /**
     * Used to update a single task
     */
    pub fn delete_task(&self, task: &Task) -> Result<()> {
        let sql = Query::delete()
            .from_table(TaskIden::Table)
            .and_where(Expr::col(TaskIden::Id).eq(task.id))
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rusqlite::Connection;

    use crate::{
        models::{setup_database, AddTask, TaskStatusEnum, UpdateTask},
        repositories::get_now,
    };

    use super::TaskRepository;

    #[test]
    fn test_crud_tasks() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let now = get_now();

        let repository = TaskRepository::create(&conn);

        // test create and fetch
        let fetched_task = repository.get_task(1)?;
        assert_eq!(None, fetched_task);

        let task = repository.create_task(AddTask {
            title: "Test Task".into(),
            info: None,
            deadline: None,
            categories: None,
            status: TaskStatusEnum::Undone,
            created_at: now,
        })?;
        assert_eq!(1, task.id);

        let fetched_task = repository.get_task(1)?;
        assert_eq!(Some(task.clone()), fetched_task);
        assert_eq!("Test Task", &fetched_task.unwrap().title);

        // test update task
        let res = repository.update_task(
            1,
            UpdateTask {
                title: None,
                info: None,
                deadline: None,
                status: None,
                created_at: None,
            },
            &now.to_string(),
        );

        if let Err(_) = res {
            assert!(true);
        } else {
            assert!(false);
        }

        repository.update_task(
            1,
            UpdateTask {
                title: Some("New Title".into()),
                info: Some(Some("Test".into())),
                deadline: Some(Some(now)),
                status: Some(TaskStatusEnum::Done),
                created_at: Some(now),
            },
            &now.to_string(),
        )?;

        let fetched_task = repository.get_task(1)?.unwrap();
        assert_eq!("New Title", &fetched_task.title);

        // test delete task
        repository.delete_task(&fetched_task)?;
        let fetched_task = repository.get_task(1)?;
        assert_eq!(None, fetched_task);

        // test create with fixed id
        let task = repository.create_task_with_id(
            1,
            AddTask {
                title: "Test Task".into(),
                info: None,
                deadline: None,
                categories: None,
                status: crate::models::TaskStatusEnum::Undone,
                created_at: now,
            },
        )?;
        assert_eq!(1, task.id);

        let fetched_task = repository.get_task(1)?;
        assert_eq!(Some(task), fetched_task);

        Ok(())
    }
}
