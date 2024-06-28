pub mod action_repository;
pub mod task_repository;

use action_repository::ActionRepository;
use anyhow::Result;
use rusqlite::Connection;
use std::time::SystemTime;
use task_repository::TaskRepository;
use time::{Date, OffsetDateTime};

use crate::{
    models::{Action, ActionEnum, ActionTypeEnum, AddTask, Task, UpdateTask},
    parse_created_at, parse_deadline,
};

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

pub fn undo_redo_operation(conn: &mut Connection, action: Action) -> Result<String> {
    let trx = conn.transaction()?;

    let task_repository = TaskRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    let result: String = match action.action {
        ActionEnum::Task {
            action_type,
            id: task_id,
            title,
            info,
            deadline,
            status,
            updated_at,
            created_at,
        } => match action_type {
            ActionTypeEnum::Create => {
                let task = task_repository
                    .get_task(task_id)?
                    .expect("Task should exist");
                task_repository.delete_task(&task)?;
                action_repository.update_action(
                    action.id,
                    ActionEnum::Task {
                        action_type: ActionTypeEnum::Delete,
                        id: task_id,
                        title: title.clone(),
                        info,
                        deadline,
                        status,
                        updated_at,
                        created_at,
                    },
                    !action.restored,
                )?;
                format!("[Undo][Task][Create] - (#{}) - [{}]", task_id, title)
            }
            ActionTypeEnum::Update => {
                let old_task = task_repository
                    .get_task(task_id)?
                    .expect("Task should exist");
                let new_task = UpdateTask {
                    title: Some(title.clone()),
                    info: Some(info),
                    deadline: Some(parse_deadline(deadline)?),
                    status: Some(status),
                    created_at: Some(parse_created_at(created_at)?),
                };
                task_repository.update_task(task_id, new_task, &updated_at)?;
                let new_action = ActionEnum::Task {
                    action_type: ActionTypeEnum::Update,
                    id: task_id,
                    title: old_task.title,
                    info: old_task.info,
                    deadline: old_task.deadline.map(|v| v.to_string()),
                    status: old_task.status,
                    updated_at: old_task.updated_at.to_string(),
                    created_at: old_task.created_at.to_string(),
                };
                action_repository.update_action(action.id, new_action, !action.restored)?;
                format!("[Undo][Task][Update] - (#{}) - [{}]", task_id, title)
            }
            ActionTypeEnum::Delete => {
                let new_task = AddTask {
                    title: title.clone(),
                    info: info.clone(),
                    deadline: parse_deadline(deadline.clone())?,
                    status: status,
                    created_at: parse_created_at(created_at.clone())?,
                };
                let task = task_repository.create_task_with_id(task_id, new_task)?;
                action_repository.update_action(
                    action.id,
                    ActionEnum::Task {
                        action_type: ActionTypeEnum::Create,
                        id: task_id,
                        title: title.clone(),
                        info,
                        deadline,
                        status,
                        updated_at,
                        created_at,
                    },
                    !action.restored,
                )?;
                format!("[Undo][Task][Created] - (#{}) - [{}]", task.id, title)
            }
        },
    };

    trx.commit()?;

    Ok(result)
}
