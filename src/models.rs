use anyhow::Result;
use rusqlite::Connection;
use sea_query::{ColumnDef, ForeignKey, Index, SqliteQueryBuilder, Table};

pub use action::*;
pub use category::*;
pub use order_by_enum::*;
pub use task::*;
pub use task_status_enum::*;

mod action;
mod category;
mod order_by_enum;
mod task;
mod task_status_enum;

/**
 * Used to initialize the database
 */
pub fn setup_database(conn: &Connection) -> Result<()> {
    let tasks_table = Table::create()
        .table(TaskIden::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(TaskIden::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(TaskIden::Title).text().not_null())
        .col(ColumnDef::new(TaskIden::Info).text())
        .col(ColumnDef::new(TaskIden::Deadline).text())
        .col(ColumnDef::new(TaskIden::Status).text().not_null())
        .col(ColumnDef::new(TaskIden::UpdatedAt).text().not_null())
        .col(ColumnDef::new(TaskIden::CreatedAt).text().not_null())
        .to_string(SqliteQueryBuilder);
    conn.execute(&tasks_table, ())?;

    let tasks_fts_table = "
        CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5 (id UNINDEXED, title, info);
    ";
    conn.execute(&tasks_fts_table, ())?;

    let idx = Index::create()
        .if_not_exists()
        .name("TASK_DEADLINE_IDX")
        .col(TaskIden::Deadline)
        .table(TaskIden::Table)
        .to_string(SqliteQueryBuilder);
    conn.execute(&idx, ())?;

    let idx = Index::create()
        .if_not_exists()
        .name("TASK_STATUS_IDX")
        .col(TaskIden::Status)
        .table(TaskIden::Table)
        .to_string(SqliteQueryBuilder);
    conn.execute(&idx, ())?;

    let idx = Index::create()
        .if_not_exists()
        .name("TASK_CREATED_AT_IDX")
        .col(TaskIden::CreatedAt)
        .table(TaskIden::Table)
        .to_string(SqliteQueryBuilder);
    conn.execute(&idx, ())?;

    let actions_table = Table::create()
        .table(ActionIden::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(ActionIden::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(ActionIden::Action)
                .blob(sea_query::BlobSize::Long)
                .not_null(),
        )
        .col(ColumnDef::new(ActionIden::Restored).boolean().not_null())
        .col(ColumnDef::new(ActionIden::CreatedAt).text().not_null())
        .to_string(SqliteQueryBuilder);
    conn.execute(&actions_table, ())?;

    let idx = Index::create()
        .if_not_exists()
        .name("ACTION_RESTORED_IDX")
        .col(ActionIden::Restored)
        .table(ActionIden::Table)
        .to_string(SqliteQueryBuilder);
    conn.execute(&idx, ())?;

    let task_categories_table = Table::create()
        .table(TaskCategoryIden::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(TaskCategoryIden::TaskId)
                .integer()
                .not_null(),
        )
        .col(ColumnDef::new(TaskCategoryIden::Category).text().not_null())
        .foreign_key(
            ForeignKey::create()
                .name("TASK_CATEGORY_TASK_FK")
                .from(TaskCategoryIden::Table, TaskCategoryIden::TaskId)
                .to(TaskIden::Table, TaskIden::Id),
        )
        .primary_key(
            Index::create()
                .col(TaskCategoryIden::TaskId)
                .col(TaskCategoryIden::Category),
        )
        .to_string(SqliteQueryBuilder);
    conn.execute(&task_categories_table, ())?;

    let idx = Index::create()
        .if_not_exists()
        .name("TASK_CATEGORY_CATEGORY_IDX")
        .col(TaskCategoryIden::Category)
        .table(TaskCategoryIden::Table)
        .to_string(SqliteQueryBuilder);
    conn.execute(&idx, ())?;

    Ok(())
}
