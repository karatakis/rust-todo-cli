use action_repository::ActionRepository;
use anyhow::Result;
use category_repository::CategoryRepository;
use rusqlite::Connection;
use std::time::SystemTime;
use task_repository::TaskRepository;
use time::{Date, OffsetDateTime};

pub mod action_repository;
pub mod category_repository;
pub mod task_repository;

use crate::{
    models::{Action, ActionEnum, ActionTypeEnum, AddTask, Task, UpdateTask},
    utils::{created_at_parser, date_parser},
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
 * Used to create a Task and create its respective Action
 */
pub fn add_task(conn: &mut Connection, task: AddTask) -> Result<Task> {
    let trx = conn.transaction()?;
    let now = get_now();

    let task_repository = TaskRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);
    let category_repository = CategoryRepository::create(&trx);

    let categories = task.categories.clone();
    let task = task_repository.create_task(task)?;

    if let Some(categories) = categories {
        category_repository.batch_create_task_categories(task.id, &categories)?;
    }

    let action = ActionEnum::Task {
        action_type: ActionTypeEnum::Create,
        id: task.id,
        title: task.title.clone(),
        info: task.info.clone(),
        deadline: task.deadline.map(|v| v.to_string()),
        status: task.status,
        updated_at: now.to_string(),
        created_at: task.created_at.to_string(),
        categories: task.categories.clone(),
    };
    action_repository.create_action(action, &now.to_string())?;

    trx.commit()?;

    Ok(task)
}

/**
 * Used to edit a Task and create its respective Action
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
        categories: None,
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
    let category_repository = CategoryRepository::create(&trx);

    task_repository.delete_task(task)?;

    // TODO fix this
    let categories = category_repository.fetch_task_categories(task.id)?;
    // TODO category_repository.batch_delete_category(category)
    let action = ActionEnum::Task {
        action_type: ActionTypeEnum::Delete,
        id: task.id,
        title: task.title.clone(),
        info: task.info.clone(),
        deadline: task.deadline.map(|v| v.to_string()),
        status: task.status,
        updated_at: task.updated_at.to_string(),
        created_at: task.created_at.to_string(),
        categories: Some(categories),
    };
    action_repository.create_action(action, &now.to_string())?;

    trx.commit()?;

    Ok(())
}

pub fn undo_redo_operation(conn: &mut Connection, action: Action) -> Result<String> {
    let trx = conn.transaction()?;

    let task_repository = TaskRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);
    let category_repository = CategoryRepository::create(&trx);

    let message = action.action.to_string();

    match action.action {
        ActionEnum::Task {
            action_type,
            id: task_id,
            title,
            info,
            deadline,
            status,
            updated_at,
            created_at,
            categories,
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
                        categories,
                    },
                    !action.restored,
                )?;
            }
            ActionTypeEnum::Update => {
                let old_task = task_repository
                    .get_task(task_id)?
                    .expect("Task should exist");
                let new_task = UpdateTask {
                    title: Some(title.clone()),
                    info: Some(info),
                    deadline: Some(deadline.map(|v| date_parser(&v)).transpose()?),
                    status: Some(status),
                    created_at: Some(created_at_parser(&created_at)?),
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
                    categories,
                };
                action_repository.update_action(action.id, new_action, !action.restored)?;
            }
            ActionTypeEnum::Delete => {
                let new_task = AddTask {
                    title: title.clone(),
                    info: info.clone(),
                    deadline: deadline.clone().map(|v| date_parser(&v)).transpose()?,
                    status: status,
                    created_at: created_at_parser(&created_at)?,
                    categories: categories.clone(),
                };
                let _ = task_repository.create_task_with_id(task_id, new_task)?;
                action_repository.update_action(
                    action.id,
                    ActionEnum::Task {
                        action_type: ActionTypeEnum::Create,
                        id: task_id,
                        title,
                        info,
                        deadline,
                        status,
                        updated_at,
                        created_at,
                        categories,
                    },
                    !action.restored,
                )?;
            }
        },
        ActionEnum::Category {
            action_type,
            category,
            task_id,
        } => match action_type {
            ActionTypeEnum::Create => {
                category_repository.delete_category(task_id, &category)?;
                action_repository.update_action(
                    action.id,
                    ActionEnum::Category {
                        action_type: ActionTypeEnum::Delete,
                        category,
                        task_id,
                    },
                    !action.restored,
                )?;
            }
            ActionTypeEnum::Update => {
                return Err(anyhow::anyhow!(
                    "Operation update not permitted for category!"
                ))
            }
            ActionTypeEnum::Delete => {
                category_repository.create_category(task_id, &category)?;
                action_repository.update_action(
                    action.id,
                    ActionEnum::Category {
                        action_type: ActionTypeEnum::Create,
                        category,
                        task_id,
                    },
                    !action.restored,
                )?;
            }
        },
        ActionEnum::RenameTaskCategory {
            old_category,
            new_category,
            task_id,
        } => {
            category_repository.rename_category(task_id, &new_category, &old_category)?;
            action_repository.update_action(
                action.id,
                ActionEnum::RenameTaskCategory {
                    new_category: old_category,
                    old_category: new_category,
                    task_id,
                },
                !action.restored,
            )?;
        }
        ActionEnum::BatchCategoryDelete { task_ids, category } => {
            category_repository.batch_create_category(&task_ids, &category)?;
            action_repository.update_action(
                action.id,
                ActionEnum::BatchCategoryDelete {
                    task_ids: task_ids,
                    category: category,
                },
                !action.restored,
            )?;
        }
        ActionEnum::BatchCategoryRename {
            old_category,
            new_category,
        } => {
            category_repository.batch_rename_category(&new_category, &old_category)?;
            action_repository.update_action(
                action.id,
                ActionEnum::BatchCategoryRename {
                    old_category: new_category,
                    new_category: old_category,
                },
                !action.restored,
            )?;
        }
    };

    trx.commit()?;

    Ok(format!("[Undo]{}", message))
}

pub fn add_category_to_task(conn: &mut Connection, task_id: i64, category: &str) -> Result<()> {
    let trx = conn.transaction()?;
    let now = get_now();

    let category_repository = CategoryRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    category_repository.create_category(task_id, category)?;

    action_repository.create_action(
        ActionEnum::Category {
            action_type: ActionTypeEnum::Create,
            category: category.to_string(),
            task_id,
        },
        &now.to_string(),
    )?;

    trx.commit()?;

    Ok(())
}

pub fn rename_task_category(
    conn: &mut Connection,
    task_id: i64,
    old_category: &str,
    new_category: &str,
) -> Result<()> {
    let trx = conn.transaction()?;
    let now = get_now();

    let category_repository = CategoryRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    category_repository.rename_category(task_id, old_category, new_category)?;

    action_repository.create_action(
        ActionEnum::RenameTaskCategory {
            old_category: old_category.to_string(),
            new_category: new_category.to_string(),
            task_id: task_id,
        },
        &now.to_string(),
    )?;

    trx.commit()?;

    Ok(())
}

pub fn remove_task_category(conn: &mut Connection, task_id: i64, category: &str) -> Result<()> {
    let trx = conn.transaction()?;
    let now = get_now();

    let category_repository = CategoryRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    category_repository.delete_category(task_id, category)?;

    action_repository.create_action(
        ActionEnum::Category {
            action_type: ActionTypeEnum::Delete,
            category: category.to_string(),
            task_id,
        },
        &now.to_string(),
    )?;

    trx.commit()?;

    Ok(())
}

pub fn batch_rename_category(
    conn: &mut Connection,
    old_category: &str,
    new_category: &str,
) -> Result<()> {
    let trx = conn.transaction()?;
    let now = get_now();

    let category_repository = CategoryRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    category_repository.batch_rename_category(old_category, new_category)?;

    action_repository.create_action(
        ActionEnum::BatchCategoryRename {
            old_category: old_category.to_string(),
            new_category: new_category.to_string(),
        },
        &now.to_string(),
    )?;

    trx.commit()?;

    Ok(())
}
pub fn batch_delete_category(conn: &mut Connection, category: &str) -> Result<()> {
    let trx = conn.transaction()?;
    let now = get_now();

    let category_repository = CategoryRepository::create(&trx);
    let action_repository = ActionRepository::create(&trx);

    let task_ids = category_repository.get_category_task_ids(category)?;

    category_repository.batch_delete_category(category)?;

    action_repository.create_action(
        ActionEnum::BatchCategoryDelete {
            task_ids,
            category: category.to_string(),
        },
        &now.to_string(),
    )?;

    trx.commit()?;

    Ok(())
}
