use clap::ValueEnum;
use rusqlite::{types::ToSqlOutput, ToSql};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TaskStatusEnum {
    Done,
    Undone,
    Archived,
}

impl ToString for TaskStatusEnum {
    fn to_string(&self) -> String {
        Into::<&str>::into(*self).to_string()
    }
}

impl Into<&'static str> for TaskStatusEnum {
    fn into(self) -> &'static str {
        match self {
            TaskStatusEnum::Done => "done",
            TaskStatusEnum::Undone => "undone",
            TaskStatusEnum::Archived => "archived",
        }
    }
}

impl Into<clap::builder::OsStr> for TaskStatusEnum {
    fn into(self) -> clap::builder::OsStr {
        Into::<&str>::into(self).into()
    }
}

impl ToSql for TaskStatusEnum {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let value: &str = (*self).into();
        Ok(ToSqlOutput::Borrowed(rusqlite::types::ValueRef::Text(
            value.as_bytes(),
        )))
    }
}
