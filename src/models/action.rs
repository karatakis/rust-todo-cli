use bincode::{config, Decode, Encode};
use rusqlite::{
    types::{FromSql, ValueRef},
    ToSql,
};
use sea_query::Iden;
use time::Date;

use super::TaskStatusEnum;

#[derive(Debug)]
pub struct Action {
    pub id: i64,
    pub action: ActionEnum,
    pub restored: bool,
    pub created_at: Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Iden)]
pub enum ActionIden {
    #[iden = "actions"]
    Table,
    Id,
    Action,
    Restored,
    CreatedAt,
}

#[derive(Debug, Encode, Decode)]
pub enum ActionTypeEnum {
    Create,
    Update,
    Delete,
}

impl ToString for ActionTypeEnum {
    fn to_string(&self) -> String {
        match self {
            ActionTypeEnum::Create => "Create".into(),
            ActionTypeEnum::Update => "Update".into(),
            ActionTypeEnum::Delete => "Delete".into(),
        }
    }
}

#[derive(Debug, Encode, Decode)]
pub enum ActionEnum {
    Task {
        action_type: ActionTypeEnum,
        id: i64,
        title: String,
        info: Option<String>,
        deadline: Option<String>,
        // TODO
        // categories: Vec<i64>,
        status: TaskStatusEnum,
        updated_at: String,
        created_at: String,
    },
}

impl ActionEnum {
    pub fn to_blob(&self) -> Vec<u8> {
        let config = config::standard();

        let data: Vec<u8> = bincode::encode_to_vec(self, config).unwrap();

        data
    }
}

impl ToString for ActionEnum {
    fn to_string(&self) -> String {
        match self {
            ActionEnum::Task {
                action_type,
                id,
                title: _,
                info: _,
                deadline: _,
                status: _,
                updated_at: _,
                created_at: _,
            } => {
                format!("[Task] - (#{}) - [{}]", id, action_type.to_string())
            }
        }
    }
}

fn action_enum_from_blob(data: &[u8]) -> ActionEnum {
    let config = config::standard();
    bincode::decode_from_slice(data, config).unwrap().0
}

impl ToSql for ActionEnum {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(self.to_blob()))
    }
}

impl FromSql for ActionEnum {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        if let ValueRef::Blob(data) = value {
            rusqlite::types::FromSqlResult::Ok(action_enum_from_blob(data))
        } else {
            rusqlite::types::FromSqlResult::Err(rusqlite::types::FromSqlError::Other(
                anyhow::anyhow!("Cannot parse blog ActionEnum").into(),
            ))
        }
    }
}
