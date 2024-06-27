mod task;
mod task_status_enum;

use anyhow::Result;
use rusqlite::Connection;
use sea_query::{ColumnDef, SqliteQueryBuilder, Table};
pub use task::*;
pub use task_status_enum::*;

pub fn setup_database(conn: &Connection) -> Result<()> {
    let task_table = Table::create()
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
        .col(ColumnDef::new(TaskIden::CreatedAt).text().not_null())
        .to_string(SqliteQueryBuilder);
    conn.execute(&task_table, ())?;

    Ok(())
}
