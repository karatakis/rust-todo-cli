use sea_query::enum_def;
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
    pub info: Option<Option<String>>,
    pub deadline: Option<Option<Date>>,
    // TODO
    // categories: Vec<TaskCategory>,
    // TODO
    // comments: Vec<TaskComment>,
    pub status: Option<TaskStatusEnum>,
    pub created_at: Option<Date>,
}
