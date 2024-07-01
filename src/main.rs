use anyhow::Result;
use clap::Parser;
use command::RootCommand;
use models::{setup_database, AddTask, QueryTaskPayload, UpdateTask};
use repositories::{
    action_repository::ActionRepository, category_repository::CategoryRepository, query_tasks,
    task_repository::TaskRepository,
};
use rusqlite::Connection;
use utils::{ask_permission, optional_date_parser};

mod command;
mod models;
mod repositories;
mod utils;

fn main() -> Result<()> {
    let matches = RootCommand::parse();

    let mut conn = Connection::open(&matches.file)?;
    setup_database(&conn)?;

    let mut conn = conn.transaction()?;

    match matches.command {
        command::RootCommandsEnum::Task { command } => match command {
            command::TaskCommandsEnum::Add {
                title,
                info,
                deadline,
                status,
                date,
                categories,
            } => {
                let task = AddTask {
                    title,
                    info,
                    deadline,
                    status,
                    created_at: date,
                    categories,
                };

                let task = repositories::add_task(&mut conn, task)?;

                println!("[Task][Create] - (#{}) - [{}]", task.id, task.title);
            }
            command::TaskCommandsEnum::Delete { id, force } => {
                let repository = TaskRepository::create(&conn);
                let task = match repository.get_task(id)? {
                    Some(task) => task,
                    None => return Err(anyhow::anyhow!("Task with id (#{}) not found!", id).into()),
                };

                let proceed = ask_permission(
                    &format!(
                        "Do you want to delete task (#{}) - [{}]? (y/N)",
                        id, task.title
                    ),
                    force,
                )?;

                if proceed {
                    repositories::delete_task(&mut conn, &task)?;
                    println!("[Task][Delete] - (#{}) - [{}]", id, task.title);
                } else {
                    println!("Operation Canceled")
                }
            }
            command::TaskCommandsEnum::Update {
                id,
                title,
                info,
                deadline,
                // categories,
                status,
                date,
                force,
            } => {
                let repository = TaskRepository::create(&conn);

                let old_task = match repository.get_task(id)? {
                    Some(existing_task) => existing_task,
                    None => return Err(anyhow::anyhow!("Task with id (#{}) not found!", id).into()),
                };

                let info = info.map(|info| if info == "" { None } else { Some(info) });
                let deadline = deadline.map(|v| optional_date_parser(&v)).transpose()?;

                let new_task = UpdateTask {
                    title,
                    info,
                    deadline,
                    status,
                    created_at: date,
                };

                let proceed = ask_permission(
                    &format!(
                        "Do you want to update task (#{}) - [{}]? (y/N)",
                        id, old_task.title
                    ),
                    force,
                )?;

                if proceed {
                    let task = repositories::edit_task(&mut conn, id, old_task, new_task)?;
                    println!("[Task][Updated] (#{}) - [{}]", id, task.title);
                } else {
                    println!("Operation Canceled")
                }
            }
            command::TaskCommandsEnum::List {
                status,
                categories,
                text,
                limit,
                sort_created_at,
                sort_updated_at,
                sort_deadline,
                sort_title,
            } => {
                let payload = QueryTaskPayload {
                    status,
                    categories,
                    text,
                    limit,
                    sort_created_at,
                    sort_updated_at,
                    sort_deadline,
                    sort_title,
                };
                let tasks = query_tasks(&conn, payload)?;
                println!("========== TASKS ==========");
                for task in tasks {
                    println!(
                        "(#{}) - [{}] - [Status: {}] - [{}]",
                        task.id,
                        task.title,
                        task.status.to_string(),
                        task.created_at
                    );
                }
            }
            command::TaskCommandsEnum::Read { id } => {
                let repository = TaskRepository::create(&conn);
                let category_repository = CategoryRepository::create(&conn);

                match repository.get_task(id)? {
                    Some(task) => {
                        let categories = category_repository.fetch_task_categories(id)?;
                        let header = format!("=== (#{}) [{}] ===", id, task.title);
                        println!("{}", header);
                        println!("{}", "=".repeat(header.len()));
                        match task.info {
                            Some(info) => {
                                println!("Info: {}", info);
                            }
                            None => {}
                        }
                        match task.deadline {
                            Some(deadline) => {
                                println!("Deadline: {}", deadline);
                            }
                            None => {}
                        }
                        println!("status: {}", task.status.to_string());
                        println!("Created At: {}", task.created_at);
                        println!("Updated At: {}", task.updated_at);
                        if categories.len() > 0 {
                            println!("Categories: {}", categories.join(", "));
                        }
                        println!("{}", "=".repeat(header.len()));
                    }
                    None => return Err(anyhow::anyhow!("Task with id (#{}) not found!", id).into()),
                }
            }
        },
        command::RootCommandsEnum::Undo { force } => {
            let repository = ActionRepository::create(&conn);

            let action = repository.get_last_unrestored_action()?;
            let proceed = ask_permission(
                &format!("Do you want to undo: {}? (y/N)", action.action.to_string()),
                force,
            )?;
            if proceed {
                let result = repositories::undo_redo_operation(&mut conn, action)?;
                println!("{}", result)
            } else {
                println!("Operation Canceled")
            }
        }
        command::RootCommandsEnum::Redo { force } => {
            let repository = ActionRepository::create(&conn);

            let action = repository.get_first_restored_action()?;
            let proceed = ask_permission(
                &format!("Do you want to redo: {}? (y/N)", action.action.to_string()),
                force,
            )?;
            if proceed {
                let result = repositories::undo_redo_operation(&mut conn, action)?;
                println!("{}", result)
            } else {
                println!("Operation Canceled")
            }
        }
        command::RootCommandsEnum::Actions { limit } => {
            let repository = ActionRepository::create(&conn);

            let actions = repository.fetch_actions(limit)?;

            println!("========== ACTIONS ==========");
            for action in actions {
                println!(
                    "(#{}) - <{}> - [Restored: {}] - [{}]",
                    action.id,
                    action.action.to_string(),
                    action.restored,
                    action.created_at
                );
            }
        }
        command::RootCommandsEnum::Category { command } => match command {
            command::CategoryCommandsEnum::List => {
                let repository = CategoryRepository::create(&conn);

                let categories = repository.all_categories()?;

                println!("========== Categories ==========");
                for category in categories {
                    println!("(#{}) - [Count: {}]", category.0, category.1);
                }
            }
            command::CategoryCommandsEnum::Add { task_id, category } => {
                repositories::add_category_to_task(&mut conn, task_id, &category)?;
                println!(
                    "[Category][Created] - (#{}) - [Task: {}]",
                    category, task_id
                );
            }
            command::CategoryCommandsEnum::Rename {
                task_id,
                old_category,
                new_category,
            } => {
                repositories::rename_task_category(
                    &mut conn,
                    task_id,
                    &old_category,
                    &new_category,
                )?;
                println!(
                    "[Category][Renamed] - (From: {}) - (To: {}) - [Task: {}]",
                    old_category, new_category, task_id
                );
            }
            command::CategoryCommandsEnum::Remove { task_id, category } => {
                repositories::remove_task_category(&mut conn, task_id, &category)?;
                println!(
                    "[Category][Removed] - (#{}) - [Task: {}]",
                    category, task_id
                );
            }
            command::CategoryCommandsEnum::BatchRename {
                old_category,
                new_category,
            } => {
                repositories::batch_rename_category(&mut conn, &old_category, &new_category)?;
                println!(
                    "[Category][Batch][Rename] - (From: {}) - (To: {})",
                    old_category, new_category
                );
            }
            command::CategoryCommandsEnum::BatchDelete { category } => {
                repositories::batch_delete_category(&mut conn, &category)?;
                println!("[Category][Batch][Delete] - (#{})", category);
            }
        },
        command::RootCommandsEnum::Housekeeping => {
            let proceed = ask_permission("This operation is going to:\n 1) Delete all actions\n 2) Delete all archived tasks\n 3) Archive all completed tasks\n(y/N)", false)?;

            if proceed {
                let (actions_deleted, tasks_deleted, tasks_updated) =
                    repositories::clean_database(&conn)?;
                println!(
                    "[Actions deleted: {}] - [Tasks deleted: {}] - [Tasks updated: {}]",
                    actions_deleted, tasks_deleted, tasks_updated
                );
            } else {
                println!("Operation Canceled")
            }
        }
    }

    conn.commit()?;

    Ok(())
}
