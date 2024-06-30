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
            .expr(Expr::col(TaskCategoryIden::Category))
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
            .column(TaskCategoryIden::Category)
            .and_where(Expr::col(TaskCategoryIden::TaskId).eq(task_id))
            .to_string(SqliteQueryBuilder);

        let records = self
            .conn
            .prepare(&sql)?
            .query_map((), |row| {
                let category: String = row.get(1)?;

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
}
