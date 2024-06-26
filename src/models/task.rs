use time::Date;

use super::TaskStatusEnum;

#[derive(Debug)]
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
