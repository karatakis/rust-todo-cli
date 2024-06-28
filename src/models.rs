use anyhow::Result;
use rusqlite::Connection;
use sea_query::{ColumnDef, SqliteQueryBuilder, Table};

pub use action::*;
pub use task::*;
pub use task_status_enum::*;

mod action;
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

    Ok(())
}
