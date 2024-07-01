use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::{
    AddTask, OrderByEnum, QueryTaskPayload, Task, TaskCategoryIden, TaskFtsIden, TaskIden,
    TaskStatusEnum, UpdateTask,
};

use super::get_now;

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

        let sql = Query::insert()
            .into_table(TaskFtsIden::Table)
            .columns([TaskFtsIden::Id, TaskFtsIden::Title, TaskFtsIden::Info])
            .values([
                id.into(),
                task.title.clone().into(),
                task.info.clone().into(),
            ])?
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

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
        let now = get_now();

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
                now.to_string().into(),
                task.created_at.into(),
            ])?
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;
        let id = self.conn.last_insert_rowid();

        let sql = Query::insert()
            .into_table(TaskFtsIden::Table)
            .columns([TaskFtsIden::Id, TaskFtsIden::Title, TaskFtsIden::Info])
            .values([
                id.into(),
                task.title.clone().into(),
                task.info.clone().into(),
            ])?
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

        Ok(Task {
            id,
            title: task.title,
            info: task.info,
            deadline: task.deadline,
            status: task.status,
            updated_at: now,
            created_at: task.created_at,
            categories: None,
        })
    }

    /**
     * Used to update a single task
     */
    pub fn update_task(&self, id: i64, new_task: UpdateTask, now: &str) -> Result<()> {
        let mut changes = 0;
        let mut changes_fts = 0;

        let mut sql = Query::update();
        let mut sql_fts = Query::update();

        sql.table(TaskIden::Table);
        sql_fts.table(TaskFtsIden::Table);

        // update only the changed fields
        if let Some(title) = new_task.title {
            sql.value(TaskIden::Title, title.clone());
            sql_fts.value(TaskFtsIden::Title, title);
            changes += 1;
            changes_fts += 1;
        }

        if let Some(info) = new_task.info {
            sql.value(TaskIden::Info, info.clone());
            sql_fts.value(TaskFtsIden::Info, info);
            changes += 1;
            changes_fts += 1;
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

            if changes_fts > 0 {
                sql_fts.and_where(Expr::col(TaskFtsIden::Id).eq(id));
                let sql = sql_fts.to_string(SqliteQueryBuilder);

                self.conn.execute(&sql, ())?;
            }

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

        let sql = Query::delete()
            .from_table(TaskFtsIden::Table)
            .and_where(Expr::col(TaskFtsIden::Id).eq(task.id))
            .to_string(SqliteQueryBuilder);
        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to query tasks
     */
    pub fn query_tasks(&self, payload: QueryTaskPayload) -> Result<Vec<Task>> {
        let mut sql = Query::select();
        sql.from(TaskIden::Table);
        sql.columns([
            TaskIden::Id,
            TaskIden::Title,
            TaskIden::Info,
            TaskIden::Deadline,
            TaskIden::Status,
            TaskIden::UpdatedAt,
            TaskIden::CreatedAt,
        ]);

        if let Some(text) = payload.text {
            let sub_query = Query::select()
                .from(TaskFtsIden::Table)
                .column(TaskFtsIden::Id)
                .and_where(Expr::col(TaskFtsIden::Table).eq(text))
                .clone();
            sql.and_where(Expr::col(TaskIden::Id).in_subquery(sub_query));
        }

        if let Some(status) = payload.status {
            sql.and_where(Expr::col(TaskIden::Status).eq(status));
        }

        if let Some(categories) = payload.categories {
            let sub_query = Query::select()
                .from(TaskCategoryIden::Table)
                .column(TaskCategoryIden::TaskId)
                .and_where(Expr::col(TaskCategoryIden::Category).is_in(categories))
                .clone();
            sql.and_where(Expr::col(TaskIden::Id).in_subquery(sub_query));
        }

        sql.limit(payload.limit);

        if let Some(sort_created_at) = payload.sort_created_at {
            match sort_created_at {
                OrderByEnum::Asc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Asc);
                }
                OrderByEnum::Desc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Desc);
                }
            }
        }
        if let Some(sort_updated_at) = payload.sort_updated_at {
            match sort_updated_at {
                OrderByEnum::Asc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Asc);
                }
                OrderByEnum::Desc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Desc);
                }
            }
        }
        if let Some(sort_deadline) = payload.sort_deadline {
            match sort_deadline {
                OrderByEnum::Asc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Asc);
                }
                OrderByEnum::Desc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Desc);
                }
            }
        }
        if let Some(sort_title) = payload.sort_title {
            match sort_title {
                OrderByEnum::Asc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Asc);
                }
                OrderByEnum::Desc => {
                    sql.order_by(TaskIden::CreatedAt, sea_query::Order::Desc);
                }
            }
        }

        let sql = sql.to_string(SqliteQueryBuilder);

        let data = self
            .conn
            .prepare(&sql)?
            .query_map((), |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    info: row.get(2)?,
                    deadline: row.get(3)?,
                    categories: None,
                    status: row.get(4)?,
                    updated_at: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(data)
    }

    /**
     * Used to delete all archived tasks
     */
    pub fn delete_archived(&self) -> Result<i64> {
        let sql = Query::select()
            .from(TaskIden::Table)
            .expr(Expr::col(TaskIden::Id).count())
            .and_where(Expr::col(TaskIden::Status).eq(TaskStatusEnum::Archived))
            .to_string(SqliteQueryBuilder);

        let count = self.conn.query_row(&sql, (), |row| Ok(row.get(0)?))?;

        let sql = Query::delete()
            .from_table(TaskIden::Table)
            .and_where(Expr::col(TaskIden::Status).eq(TaskStatusEnum::Archived))
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(count)
    }

    /**
     * Used to archive all completed tasks
     */
    pub fn archive_tasks(&self) -> Result<i64> {
        let sql = Query::select()
            .from(TaskIden::Table)
            .expr(Expr::col(TaskIden::Id).count())
            .and_where(Expr::col(TaskIden::Status).eq(TaskStatusEnum::Done))
            .to_string(SqliteQueryBuilder);

        let count = self.conn.query_row(&sql, (), |row| Ok(row.get(0)?))?;

        let sql = Query::update()
            .table(TaskIden::Table)
            .and_where(Expr::col(TaskIden::Status).eq(TaskStatusEnum::Done))
            .value(TaskIden::Status, TaskStatusEnum::Archived)
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
