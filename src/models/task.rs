use sea_query::Iden;
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
    pub updated_at: Date,
    pub created_at: Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Iden)]
pub enum TaskIden {
    #[iden = "tasks"]
    Table,
    Id,
    Title,
    Info,
    Deadline,
    Status,
    UpdatedAt,
    CreatedAt,
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
