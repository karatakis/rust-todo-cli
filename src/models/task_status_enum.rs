use bincode::{Decode, Encode};
use clap::ValueEnum;
use rusqlite::{
    types::{FromSql, ToSqlOutput},
    ToSql,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Encode, Decode)]
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

impl FromSql for TaskStatusEnum {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let value: &str = value.as_str()?;

        match value {
            "done" => rusqlite::types::FromSqlResult::Ok(TaskStatusEnum::Done),
            "undone" => rusqlite::types::FromSqlResult::Ok(TaskStatusEnum::Undone),
            "archived" => rusqlite::types::FromSqlResult::Ok(TaskStatusEnum::Archived),
            _ => rusqlite::types::FromSqlResult::Err(rusqlite::types::FromSqlError::Other(
                anyhow::anyhow!("Could not convert '{}' to TaskStatusEnum", value).into(),
            )),
        }
    }
}

impl Into<sea_query::SimpleExpr> for TaskStatusEnum {
    fn into(self) -> sea_query::SimpleExpr {
        self.to_string().into()
    }
}
