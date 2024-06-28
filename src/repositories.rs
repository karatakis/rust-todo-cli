pub mod action_repository;
pub mod task_repository;

use action_repository::ActionRepository;
use anyhow::Result;
use rusqlite::Connection;
use std::time::SystemTime;
use task_repository::TaskRepository;
use time::{Date, OffsetDateTime};

use crate::models::{ActionEnum, ActionTypeEnum, AddTask, Task, UpdateTask};

/**
 * Used to get current date
 */
fn get_now() -> Date {
    let now = SystemTime::now();
    let now = OffsetDateTime::from(now).date();
    now
}

/**
 * Used to create a Task and its respective Action
 */
pub fn add_task(conn: &mut Connection, task: AddTask) -> Result<Task> {
    let trx = conn.transaction()?;
    let now = get_now();

    let task_repository = TaskRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    let task = task_repository.create_task(task)?;

    let action = ActionEnum::Task {
        action_type: ActionTypeEnum::Create,
        id: task.id,
        title: task.title.clone(),
        info: task.info.clone(),
        deadline: task.deadline.map(|v| v.to_string()),
        status: task.status,
        updated_at: now.to_string(),
        created_at: task.created_at.to_string(),
    };
    action_repository.create_action(action, &now.to_string())?;

    trx.commit()?;

    Ok(task)
}

/**
 * Used to edit a Task and its respective Action
 */
pub fn edit_task(
    conn: &mut Connection,
    id: i64,
    old_task: Task,
    new_task: UpdateTask,
) -> Result<Task> {
    let trx = conn.transaction()?;
    let now = get_now();

    let task_repository = TaskRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    task_repository.update_task(id, new_task, &now.to_string())?;

    let action = ActionEnum::Task {
        action_type: ActionTypeEnum::Update,
        id: old_task.id,
        title: old_task.title.clone(),
        info: old_task.info.clone(),
        deadline: old_task.deadline.map(|v| v.to_string()),
        status: old_task.status,
        updated_at: old_task.updated_at.to_string(),
        created_at: old_task.created_at.to_string(),
    };
    action_repository.create_action(action, &now.to_string())?;

    let task = task_repository.get_task(id)?.expect("Task should exist");

    trx.commit()?;

    Ok(task)
}

/**
 * Used to delete a Task and its respective Action
 */
pub fn delete_task(conn: &mut Connection, task: &Task) -> Result<()> {
    let trx = conn.transaction()?;
    let now = get_now();

    let task_repository = TaskRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    task_repository.delete_task(task)?;

    let action = ActionEnum::Task {
        action_type: ActionTypeEnum::Delete,
        id: task.id,
        title: task.title.clone(),
        info: task.info.clone(),
        deadline: task.deadline.map(|v| v.to_string()),
        status: task.status,
        updated_at: task.updated_at.to_string(),
        created_at: task.created_at.to_string(),
    };
    action_repository.create_action(action, &now.to_string())?;

    trx.commit()?;

    Ok(())
}
