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
    models::{Action, ActionEnum, ActionTypeEnum, AddTask, QueryTaskPayload, Task, UpdateTask},
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
pub fn add_task(conn: &Connection, task: AddTask) -> Result<Task> {
    let now = get_now();

    let task_repository = TaskRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);
    let category_repository = CategoryRepository::create(&conn);

    let categories = task.categories.clone();
    let task = task_repository.create_task(task)?;

    if let Some(categories) = &categories {
        category_repository.batch_create_task_categories(task.id, categories)?;
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
        categories,
    };
    action_repository.create_action(action, &now.to_string())?;

    Ok(task)
}

/**
 * Used to edit a Task and create its respective Action
 */
pub fn edit_task(conn: &Connection, id: i64, old_task: Task, new_task: UpdateTask) -> Result<Task> {
    let now = get_now();

    let task_repository = TaskRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);

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

    Ok(task)
}

/**
 * Used to delete a Task and its respective Action
 */
pub fn delete_task(conn: &Connection, task: &Task) -> Result<()> {
    let now = get_now();

    let task_repository = TaskRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);
    let category_repository = CategoryRepository::create(&conn);

    let categories = category_repository.fetch_task_categories(task.id)?;

    category_repository.delete_task_categories(task.id)?;

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
        categories: Some(categories),
    };
    action_repository.create_action(action, &now.to_string())?;

    Ok(())
}

/**
 * Used to undo/redo a performed logged action
 */
pub fn undo_redo_operation(conn: &Connection, action: Action) -> Result<String> {
    let task_repository = TaskRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);
    let category_repository = CategoryRepository::create(&conn);

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
                category_repository.delete_task_categories(task_id)?;
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
                    categories: None,
                };
                let _ = task_repository.create_task_with_id(task_id, new_task)?;
                if let Some(categories) = &categories {
                    category_repository.batch_create_task_categories(task_id, categories)?;
                }
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
            if action.restored {
                category_repository.batch_delete_category(&category)?;
            } else {
                category_repository.batch_create_category(&task_ids, &category)?;
            }
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

    Ok(format!("[Undo]{}", message))
}

/**
 * Used to add a new category on the task
 */
pub fn add_category_to_task(conn: &Connection, task_id: i64, category: &str) -> Result<()> {
    let now = get_now();

    let category_repository = CategoryRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);

    category_repository.create_category(task_id, category)?;

    action_repository.create_action(
        ActionEnum::Category {
            action_type: ActionTypeEnum::Create,
            category: category.to_string(),
            task_id,
        },
        &now.to_string(),
    )?;

    Ok(())
}

/**
 * Used to rename a category for a task
 */
pub fn rename_task_category(
    conn: &Connection,
    task_id: i64,
    old_category: &str,
    new_category: &str,
) -> Result<()> {
    let now = get_now();

    let category_repository = CategoryRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);

    category_repository.rename_category(task_id, old_category, new_category)?;

    action_repository.create_action(
        ActionEnum::RenameTaskCategory {
            old_category: old_category.to_string(),
            new_category: new_category.to_string(),
            task_id: task_id,
        },
        &now.to_string(),
    )?;

    Ok(())
}

/**
 * Used to remove a category from a task
 */
pub fn remove_task_category(conn: &Connection, task_id: i64, category: &str) -> Result<()> {
    let now = get_now();

    let category_repository = CategoryRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);

    category_repository.delete_category(task_id, category)?;

    action_repository.create_action(
        ActionEnum::Category {
            action_type: ActionTypeEnum::Delete,
            category: category.to_string(),
            task_id,
        },
        &now.to_string(),
    )?;

    Ok(())
}

/**
 * used to batch rename a category
 */
pub fn batch_rename_category(
    conn: &Connection,
    old_category: &str,
    new_category: &str,
) -> Result<()> {
    let now = get_now();

    let category_repository = CategoryRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);

    category_repository.batch_rename_category(old_category, new_category)?;

    action_repository.create_action(
        ActionEnum::BatchCategoryRename {
            old_category: old_category.to_string(),
            new_category: new_category.to_string(),
        },
        &now.to_string(),
    )?;

    Ok(())
}

/**
 * Used to batch delete a category
 */
pub fn batch_delete_category(conn: &Connection, category: &str) -> Result<()> {
    let now = get_now();

    let category_repository = CategoryRepository::create(&conn);
    let action_repository = ActionRepository::create(&conn);

    let task_ids = category_repository.get_category_task_ids(category)?;

    category_repository.batch_delete_category(category)?;

    action_repository.create_action(
        ActionEnum::BatchCategoryDelete {
            task_ids,
            category: category.to_string(),
        },
        &now.to_string(),
    )?;

    Ok(())
}

/**
 * Used to query tasks
 */
pub fn query_tasks(conn: &Connection, payload: QueryTaskPayload) -> Result<Vec<Task>> {
    let task_repository = TaskRepository::create(&conn);
    task_repository.query_tasks(payload)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rusqlite::Connection;

    use crate::{
        models::{setup_database, ActionEnum, ActionTypeEnum, AddTask, TaskStatusEnum, UpdateTask},
        repositories::{add_category_to_task, category_repository::CategoryRepository, edit_task},
        utils::date_parser,
    };

    use super::{
        action_repository::ActionRepository, add_task, batch_delete_category,
        batch_rename_category, delete_task, get_now, remove_task_category, rename_task_category,
        task_repository::TaskRepository, undo_redo_operation,
    };

    #[test]
    fn test_crud_task() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let now = get_now();
        let task_repository = TaskRepository::create(&conn);
        let action_repository = ActionRepository::create(&conn);
        let category_repository = CategoryRepository::create(&conn);

        // test create
        let task = {
            let task = AddTask {
                title: "Demo Task".into(),
                info: Some("Some info".into()),
                deadline: Some(date_parser("2024-01-01")?),
                categories: Some(vec!["one".into(), "two".into()]),
                status: TaskStatusEnum::Undone,
                created_at: now,
            };
            add_task(&conn, task)?;

            let task = task_repository.get_task(1)?.unwrap();
            assert_eq!("Demo Task", &task.title);
            assert_eq!(&TaskStatusEnum::Undone, &task.status);

            let categories = category_repository.fetch_task_categories(1)?;
            assert!(categories.contains(&"one".to_string()));
            assert!(categories.contains(&"two".to_string()));

            task
        };

        // test undo/redo create
        {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Create, action_type);
                    assert_eq!("Demo Task", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;

            let task = task_repository.get_task(1)?;
            if let None = task {
                assert!(true)
            } else {
                assert!(false)
            }

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Delete, action_type);
                    assert_eq!("Demo Task", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Create, action_type);
                    assert_eq!("Demo Task", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
        }

        // test edit
        {
            edit_task(
                &conn,
                1,
                task,
                UpdateTask {
                    title: Some("New Title".into()),
                    info: None,
                    deadline: None,
                    status: None,
                    created_at: None,
                },
            )?;

            let task = task_repository.get_task(1)?.unwrap();
            assert_eq!("New Title", &task.title);
            assert_eq!(&TaskStatusEnum::Undone, &task.status);
        };

        // test undo / redo edit
        let task = {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Update, action_type);
                    assert_eq!("Demo Task", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;
            let task = task_repository.get_task(1)?.unwrap();
            assert_eq!("Demo Task", &task.title);
            assert_eq!(&TaskStatusEnum::Undone, &task.status);

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Create, action_type);
                    assert_eq!("Demo Task", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Update, action_type);
                    assert_eq!("New Title", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;

            let task = task_repository.get_task(1)?.unwrap();
            assert_eq!("New Title", &task.title);
            assert_eq!(&TaskStatusEnum::Undone, &task.status);

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Update, action_type);
                    assert_eq!("Demo Task", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
            task
        };

        // test delete
        {
            delete_task(&conn, &task)?;
            let task = task_repository.get_task(1)?;
            if let None = task {
                assert!(true)
            } else {
                assert!(false)
            }
        }

        // undo / redo delete
        {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Delete, action_type);
                    assert_eq!("New Title", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;

            let task = task_repository.get_task(1)?.unwrap();
            assert_eq!("New Title", &task.title);
            assert_eq!(&TaskStatusEnum::Undone, &task.status);

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Create, action_type);
                    assert_eq!("New Title", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Task {
                    action_type,
                    id: _,
                    title,
                    info: _,
                    deadline: _,
                    categories: _,
                    status: _,
                    updated_at: _,
                    created_at: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Delete, action_type);
                    assert_eq!("New Title", title);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
        }

        Ok(())
    }

    #[test]
    fn test_category_operations() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        setup_database(&conn)?;

        let now = get_now();
        let category_repository = CategoryRepository::create(&conn);
        let action_repository = ActionRepository::create(&conn);

        let task_one = AddTask {
            title: "Task One".into(),
            info: Some("Some info".into()),
            deadline: Some(date_parser("2024-01-01")?),
            categories: Some(vec!["one".into(), "two".into()]),
            status: TaskStatusEnum::Undone,
            created_at: now,
        };
        let task_one = add_task(&conn, task_one)?;

        let task_two = AddTask {
            title: "Task Two".into(),
            info: Some("Some info".into()),
            deadline: Some(date_parser("2024-01-01")?),
            categories: Some(vec!["two".into(), "three".into()]),
            status: TaskStatusEnum::Undone,
            created_at: now,
        };
        let task_two = add_task(&conn, task_two)?;

        // test add category to task
        {
            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(!categories.contains(&"test".to_string()));
            add_category_to_task(&conn, task_two.id, "test")?;
            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(3, categories.len());
            assert!(categories.contains(&"test".to_string()));
        }

        // undo redo operation
        {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Category {
                    action_type,
                    category,
                    task_id: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Create, action_type);
                    assert_eq!("test", category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(!categories.contains(&"test".to_string()));

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::Category {
                    action_type,
                    category,
                    task_id: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Delete, action_type);
                    assert_eq!("test", category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(3, categories.len());
            assert!(categories.contains(&"test".to_string()));

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::Category {
                    action_type,
                    category,
                    task_id: _,
                } => {
                    assert_eq!(&ActionTypeEnum::Create, action_type);
                    assert_eq!("test", category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
        }

        // test rename task
        {
            rename_task_category(&conn, task_two.id, "test", "tost")?;
            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(3, categories.len());
            assert!(!categories.contains(&"test".to_string()));
            assert!(categories.contains(&"tost".to_string()));
        }

        // undo redo operation
        {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::RenameTaskCategory {
                    old_category,
                    new_category,
                    task_id: _,
                } => {
                    assert_eq!("test", old_category);
                    assert_eq!("tost", new_category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(3, categories.len());
            assert!(categories.contains(&"test".to_string()));
            assert!(!categories.contains(&"tost".to_string()));

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::RenameTaskCategory {
                    old_category,
                    new_category,
                    task_id: _,
                } => {
                    assert_eq!("tost", old_category);
                    assert_eq!("test", new_category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(3, categories.len());
            assert!(!categories.contains(&"test".to_string()));
            assert!(categories.contains(&"tost".to_string()));

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::RenameTaskCategory {
                    old_category,
                    new_category,
                    task_id: _,
                } => {
                    assert_eq!("test", old_category);
                    assert_eq!("tost", new_category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
        }

        // test remove task
        {
            remove_task_category(&conn, task_two.id, "tost")?;
            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(!categories.contains(&"test".to_string()));
            assert!(!categories.contains(&"tost".to_string()));
            assert!(categories.contains(&"two".to_string()));
            assert!(categories.contains(&"three".to_string()));
        }

        // test batch rename category
        {
            batch_rename_category(&conn, "two", "too")?;

            let categories = category_repository.fetch_task_categories(task_one.id)?;
            assert_eq!(2, categories.len());
            assert!(categories.contains(&"too".to_string()));
            assert!(categories.contains(&"one".to_string()));

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(categories.contains(&"too".to_string()));
            assert!(categories.contains(&"three".to_string()));
        }

        // undo redo operation
        {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::BatchCategoryRename {
                    old_category,
                    new_category,
                } => {
                    assert_eq!("two", old_category);
                    assert_eq!("too", new_category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(categories.contains(&"two".to_string()));
            assert!(!categories.contains(&"too".to_string()));

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::BatchCategoryRename {
                    old_category,
                    new_category,
                } => {
                    assert_eq!("too", old_category);
                    assert_eq!("two", new_category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(!categories.contains(&"two".to_string()));
            assert!(categories.contains(&"too".to_string()));

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::BatchCategoryRename {
                    old_category,
                    new_category,
                } => {
                    assert_eq!("two", old_category);
                    assert_eq!("too", new_category);
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
        }

        // test batch delete category
        {
            batch_delete_category(&conn, "too")?;

            let categories = category_repository.fetch_task_categories(task_one.id)?;
            assert_eq!(1, categories.len());
            assert!(categories.contains(&"one".to_string()));

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(1, categories.len());
            assert!(categories.contains(&"three".to_string()));
        }

        // undo redo operation
        {
            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::BatchCategoryDelete { category, task_ids } => {
                    assert_eq!("too", category);
                    assert!(task_ids.contains(&1));
                    assert!(task_ids.contains(&2));
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_one.id)?;
            assert_eq!(2, categories.len());
            assert!(categories.contains(&"one".to_string()));
            assert!(categories.contains(&"too".to_string()));

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(2, categories.len());
            assert!(categories.contains(&"three".to_string()));
            assert!(categories.contains(&"too".to_string()));

            let action = action_repository.get_first_restored_action()?;
            match &action.action {
                ActionEnum::BatchCategoryDelete { category, task_ids } => {
                    assert_eq!("too", category);
                    assert!(task_ids.contains(&1));
                    assert!(task_ids.contains(&2));
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(true, action.restored);

            undo_redo_operation(&conn, action)?;

            let categories = category_repository.fetch_task_categories(task_one.id)?;
            assert_eq!(1, categories.len());
            assert!(categories.contains(&"one".to_string()));

            let categories = category_repository.fetch_task_categories(task_two.id)?;
            assert_eq!(1, categories.len());
            assert!(categories.contains(&"three".to_string()));

            let action = action_repository.get_last_unrestored_action()?;
            match &action.action {
                ActionEnum::BatchCategoryDelete { category, task_ids } => {
                    assert_eq!("too", category);
                    assert!(task_ids.contains(&1));
                    assert!(task_ids.contains(&2));
                }
                _ => return Err(anyhow::anyhow!("Should not reach this point")),
            };
            assert_eq!(false, action.restored);
        }

        Ok(())
    }
}
