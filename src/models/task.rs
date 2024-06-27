use rusqlite::types::Value;
use sea_query::enum_def;
use std::collections::HashMap;
use time::Date;

use super::TaskStatusEnum;

#[derive(Debug)]
#[enum_def]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub info: Option<String>,
    pub deadline: Option<Date>,
    // TODO
    // categories: Vec<TaskCategory>,
    // TODO
    // comments: Vec<TaskComment>,
    pub status: TaskStatusEnum,
    pub created_at: Date,
}

#[derive(Debug)]
pub struct AddTask {
    pub title: String,
    pub info: Option<String>,
    pub deadline: Option<Date>,
    // TODO
    // categories: Vec<TaskCategory>,
    // TODO
    // comments: Vec<TaskComment>,
    pub status: TaskStatusEnum,
    pub created_at: Date,
}

#[derive(Debug)]
pub struct UpdateTask {
    pub id: i64,
    pub title: Option<String>,
    pub info: Option<String>,
    pub deadline: Option<Date>,
    // TODO
    // categories: Vec<TaskCategory>,
    // TODO
    // comments: Vec<TaskComment>,
    pub status: Option<TaskStatusEnum>,
    pub created_at: Option<Date>,
}

impl UpdateTask {
    pub fn to_update_hashmap<'a>(self) -> HashMap<&'a str, Value> {
        let mut updates: HashMap<&str, Value> = HashMap::new();

        if let Some(title) = self.title {
            updates.insert("title", Value::Text(title));
        }
        if let Some(info) = self.info {
            updates.insert("info", Value::Text(info));
        }
        if let Some(deadline) = self.deadline {
            updates.insert("deadline", Value::Text(deadline.to_string()));
        }
        if let Some(status) = &self.status {
            updates.insert("status", Value::Text(status.to_string()));
        }
        if let Some(created_at) = &self.created_at {
            updates.insert("created_at", Value::Text(created_at.to_string()));
        }

        updates
    }
}
