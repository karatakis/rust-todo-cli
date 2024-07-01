use anyhow::Result;
use rusqlite::Connection;
use sea_query::{Expr, Query, SqliteQueryBuilder};

use crate::models::TaskCategoryIden;

/**
 * Category database repository
 */
pub struct CategoryRepository<'a> {
    conn: &'a Connection,
}

impl<'a> CategoryRepository<'a> {
    /**
     * Used to initialize the repository
     */
    pub fn create(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /**
     * Used to create a category for a task
     */
    pub fn create_category(&self, task_id: i64, category: &str) -> Result<()> {
        let sql = Query::insert()
            .into_table(TaskCategoryIden::Table)
            .columns([TaskCategoryIden::TaskId, TaskCategoryIden::Category])
            .values([task_id.into(), category.into()])?
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to create batch categories for a task
     */
    pub fn batch_create_task_categories(
        &self,
        task_id: i64,
        categories: &Vec<String>,
    ) -> Result<()> {
        let mut sql = Query::insert();

        sql.into_table(TaskCategoryIden::Table)
            .columns([TaskCategoryIden::TaskId, TaskCategoryIden::Category]);

        for category in categories {
            sql.values([task_id.into(), category.into()])?;
        }

        let sql = sql.to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to fetch all categories
     */
    pub fn all_categories(&self) -> Result<Vec<(String, i64)>> {
        let sql = Query::select()
            .expr(Expr::col(TaskCategoryIden::TaskId).count())
            .column(TaskCategoryIden::Category)
            .from(TaskCategoryIden::Table)
            .group_by_col(TaskCategoryIden::Category)
            .to_string(SqliteQueryBuilder);

        let records: Vec<_> = self
            .conn
            .prepare(&sql)?
            .query_map((), |row| {
                let count: i64 = row.get(0)?;
                let category: String = row.get(1)?;

                Ok((category, count))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /**
     * Used to fetch all categories for a task
     */
    pub fn fetch_task_categories(&self, task_id: i64) -> Result<Vec<String>> {
        let sql = Query::select()
            .from(TaskCategoryIden::Table)
            .column(TaskCategoryIden::Category)
            .and_where(Expr::col(TaskCategoryIden::TaskId).eq(task_id))
            .to_string(SqliteQueryBuilder);

        let records = self
            .conn
            .prepare(&sql)?
            .query_map((), |row| {
                let category: String = row.get(0)?;

                Ok(category)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /**
     * Used to fetch a category for a task
     */
    pub fn fetch_category(&self, task_id: i64, category: &str) -> Result<Option<String>> {
        let sql = Query::select()
            .from(TaskCategoryIden::Table)
            .column(TaskCategoryIden::Category)
            .and_where(Expr::col(TaskCategoryIden::TaskId).eq(task_id))
            .and_where(Expr::col(TaskCategoryIden::Category).eq(category))
            .to_string(SqliteQueryBuilder);

        let category = self.conn.query_row(&sql, (), |row| {
            let category: String = row.get(0)?;
            Ok(category)
        });

        match category {
            Ok(category) => Ok(Some(category)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /**
     * Used to delete a category for a task
     */
    pub fn delete_category(&self, task_id: i64, category: &str) -> Result<()> {
        let task = self.fetch_category(task_id, category)?;

        if let None = task {
            return Err(anyhow::anyhow!("Category not found (#{})", category));
        }

        let sql = Query::delete()
            .from_table(TaskCategoryIden::Table)
            .and_where(Expr::col(TaskCategoryIden::TaskId).eq(task_id))
            .and_where(Expr::col(TaskCategoryIden::Category).eq(category))
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to rename a category for a task
     */
    pub fn rename_category(
        &self,
        task_id: i64,
        old_category: &str,
        new_category: &str,
    ) -> Result<()> {
        let task = self.fetch_category(task_id, old_category)?;

        if let None = task {
            return Err(anyhow::anyhow!("Category not found (#{})", old_category));
        }

        let sql = Query::update()
            .table(TaskCategoryIden::Table)
            .value(TaskCategoryIden::Category, new_category)
            .and_where(Expr::col(TaskCategoryIden::TaskId).eq(task_id))
            .and_where(Expr::col(TaskCategoryIden::Category).eq(old_category))
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to batch create a category and associate it with tasks
     */
    pub fn batch_create_category(&self, task_ids: &Vec<i64>, category: &str) -> Result<()> {
        let mut sql = Query::insert();

        sql.into_table(TaskCategoryIden::Table)
            .columns([TaskCategoryIden::TaskId, TaskCategoryIden::Category]);

        for id in task_ids {
            sql.values([(*id).into(), category.into()])?;
        }

        let sql = sql.to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to batch delete a category
     */
    pub fn batch_delete_category(&self, category: &str) -> Result<()> {
        let sql = Query::delete()
            .from_table(TaskCategoryIden::Table)
            .and_where(Expr::col(TaskCategoryIden::Category).eq(category))
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to fetch all task ids associated with the category
     */
    pub fn get_category_task_ids(&self, category: &str) -> Result<Vec<i64>> {
        let sql = Query::select()
            .from(TaskCategoryIden::Table)
            .column(TaskCategoryIden::TaskId)
            .and_where(Expr::col(TaskCategoryIden::Category).eq(category))
            .to_string(SqliteQueryBuilder);

        let result = self
            .conn
            .prepare(&sql)?
            .query_map((), |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(result)
    }

    /**
     * Used to batch rename a category
     */
    pub fn batch_rename_category(&self, old_category: &str, new_category: &str) -> Result<()> {
        let sql = Query::update()
            .table(TaskCategoryIden::Table)
            .value(TaskCategoryIden::Category, new_category)
            .and_where(Expr::col(TaskCategoryIden::Category).eq(old_category))
            .to_string(SqliteQueryBuilder);

        self.conn.execute(&sql, ())?;

        Ok(())
    }

    /**
     * Used to batch delete task categories
     * TODO: test
     */
    pub fn delete_task_categories(&self, task_id: i64) -> Result<()> {
        let sql = Query::delete()
            .from_table(TaskCategoryIden::Table)
            .and_where(Expr::col(TaskCategoryIden::TaskId).eq(task_id))
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
        models::{setup_database, AddTask, TaskStatusEnum},
        repositories::{get_now, task_repository::TaskRepository},
    };

    use super::CategoryRepository;

    #[test]
    fn test_singular_crud_category() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let now = get_now();

        let repository = CategoryRepository::create(&conn);
        let task_repository = TaskRepository::create(&conn);

        // prepare
        let task = task_repository.create_task(AddTask {
            title: String::from("Demo"),
            info: None,
            deadline: None,
            categories: None,
            status: TaskStatusEnum::Undone,
            created_at: now,
        })?;
        let categories = repository.fetch_task_categories(task.id)?;
        assert_eq!(0, categories.len());

        // test create
        repository.create_category(task.id, "test_category")?;
        let data = repository.fetch_category(task.id, "test_category")?;
        assert_eq!(Some("test_category".into()), data);

        // test rename
        let data = repository.fetch_category(task.id, "dummy")?;
        assert_eq!(None, data);
        repository.rename_category(task.id, "test_category", "dummy")?;
        let data = repository.fetch_category(task.id, "dummy")?;
        assert_eq!(Some("dummy".into()), data);
        let data = repository.fetch_category(task.id, "test_category")?;
        assert_eq!(None, data);

        // test rename not found
        if let Err(_) = repository.rename_category(task.id, "404", "impossible") {
            assert!(true);
        } else {
            assert!(false);
        }

        // test delete not found
        if let Err(_) = repository.delete_category(task.id, "404") {
            assert!(true);
        } else {
            assert!(false);
        }

        // test delete
        repository.delete_category(task.id, "dummy")?;
        let data = repository.fetch_category(task.id, "dummy")?;
        assert_eq!(None, data);
        let data = repository.fetch_category(task.id, "test_category")?;
        assert_eq!(None, data);

        Ok(())
    }

    #[test]
    fn test_batch_crud_category() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let now = get_now();

        let repository = CategoryRepository::create(&conn);
        let task_repository = TaskRepository::create(&conn);

        // prepare
        let task_one = task_repository.create_task(AddTask {
            title: String::from("Task One"),
            info: None,
            deadline: None,
            categories: None,
            status: TaskStatusEnum::Undone,
            created_at: now,
        })?;
        let categories = repository.fetch_task_categories(task_one.id)?;
        assert_eq!(0, categories.len());

        let task_two = task_repository.create_task(AddTask {
            title: String::from("Task Two"),
            info: None,
            deadline: None,
            categories: None,
            status: TaskStatusEnum::Undone,
            created_at: now,
        })?;
        let categories = repository.fetch_task_categories(task_two.id)?;
        assert_eq!(0, categories.len());

        let task_three = task_repository.create_task(AddTask {
            title: String::from("Task Three"),
            info: None,
            deadline: None,
            categories: None,
            status: TaskStatusEnum::Undone,
            created_at: now,
        })?;
        let categories = repository.fetch_task_categories(task_three.id)?;
        assert_eq!(0, categories.len());

        let mut task_one_categories: Vec<String> = vec!["one".into(), "two".into(), "three".into()];
        task_one_categories.sort();

        let mut task_two_categories: Vec<String> =
            vec!["three".into(), "four".into(), "five".into()];
        task_two_categories.sort();

        // test create
        repository.batch_create_task_categories(task_one.id, &task_one_categories)?;
        let mut categories = repository.fetch_task_categories(task_one.id)?;
        categories.sort();
        assert_eq!(task_one_categories, categories);
        let categories = repository.fetch_task_categories(task_two.id)?;
        assert_eq!(0, categories.len());

        repository.batch_create_task_categories(task_two.id, &task_two_categories)?;
        let categories = repository.fetch_task_categories(task_one.id)?;
        assert_eq!(task_one_categories, categories);
        let categories = repository.fetch_task_categories(task_two.id)?;
        assert_eq!(task_two_categories, categories);
        let categories = repository.fetch_task_categories(task_three.id)?;
        assert_eq!(0, categories.len());

        // test get all
        let mut all_categories = repository.all_categories()?;
        all_categories.sort();
        let mut expected = vec![
            ("one".into(), 1),
            ("two".into(), 1),
            ("three".into(), 2),
            ("four".into(), 1),
            ("five".into(), 1),
        ];
        expected.sort();
        assert_eq!(expected, all_categories);

        // test create
        repository.batch_create_category(&vec![1, 2, 3], "test")?;
        let mut all_categories = repository.all_categories()?;
        all_categories.sort();
        let mut expected = vec![
            ("one".into(), 1),
            ("two".into(), 1),
            ("three".into(), 2),
            ("four".into(), 1),
            ("five".into(), 1),
            ("test".into(), 3),
        ];
        expected.sort();
        assert_eq!(expected, all_categories);

        let categories = repository.fetch_task_categories(task_three.id)?;
        assert_eq!(vec!["test"], categories);

        // test get ids
        let mut ids = repository.get_category_task_ids("test")?;
        ids.sort();
        assert_eq!(vec![1, 2, 3], ids);

        // test rename
        repository.batch_rename_category("test", "tost")?;
        let mut all_categories = repository.all_categories()?;
        all_categories.sort();
        let mut expected = vec![
            ("one".into(), 1),
            ("two".into(), 1),
            ("three".into(), 2),
            ("four".into(), 1),
            ("five".into(), 1),
            ("tost".into(), 3),
        ];
        expected.sort();
        assert_eq!(expected, all_categories);
        let categories = repository.fetch_task_categories(task_three.id)?;
        assert_eq!(vec!["tost"], categories);

        let mut ids = repository.get_category_task_ids("tost")?;
        ids.sort();
        assert_eq!(vec![1, 2, 3], ids);
        let mut ids = repository.get_category_task_ids("test")?;
        ids.sort();
        assert_eq!(0, ids.len());

        // test delete
        repository.batch_delete_category("tost")?;
        let mut all_categories = repository.all_categories()?;
        all_categories.sort();
        let mut expected = vec![
            ("one".into(), 1),
            ("two".into(), 1),
            ("three".into(), 2),
            ("four".into(), 1),
            ("five".into(), 1),
        ];
        expected.sort();
        assert_eq!(expected, all_categories);

        let categories = repository.fetch_task_categories(task_three.id)?;
        assert_eq!(0, categories.len());

        let mut ids = repository.get_category_task_ids("tost")?;
        ids.sort();
        assert_eq!(0, ids.len());

        let mut ids = repository.get_category_task_ids("test")?;
        ids.sort();
        assert_eq!(0, ids.len());

        Ok(())
    }
}
