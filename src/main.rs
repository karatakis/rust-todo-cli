use anyhow::Result;
use clap::Parser;
use command::RootCommand;
use models::{setup_database, AddTask, UpdateTask};
use repositories::{
    action_repository::ActionRepository, add_task, delete_task, edit_task,
    task_repository::TaskRepository, undo_redo_operation,
};
use rusqlite::Connection;
use time::{macros::format_description, Date};

mod command;
mod models;
mod repositories;

fn main() -> Result<()> {
    let matches = RootCommand::parse();

    let mut conn = Connection::open(&matches.file)?;
    setup_database(&conn)?;

    match matches.command {
        command::RootCommandsEnum::Task { command } => match command {
            command::TaskCommandsEnum::Add {
                title,
                info,
                deadline,
                status,
                date,
            } => {
                let deadline = parse_deadline(deadline)?;
                let date = parse_date_now(&date);
                let created_at = parse_created_at(date)?;
                let task = AddTask {
                    title,
                    info,
                    deadline,
                    status,
                    created_at,
                };

                let task = add_task(&mut conn, task)?;

                println!("[TASK CREATED] (#{}) - [{}]", task.id, task.title);
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
                    delete_task(&mut conn, &task)?;
                    println!("[TASK DELETED] (#{}) - [{}]", id, task.title);
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
                let deadline = deadline
                    .map(|deadline| -> anyhow::Result<Option<Date>> {
                        if deadline == "" {
                            Ok(None)
                        } else {
                            parse_deadline(Some(deadline))
                        }
                    })
                    .transpose()?;
                let created_at = date.map(parse_created_at).transpose()?;
                let new_task = UpdateTask {
                    title,
                    info,
                    deadline,
                    status,
                    created_at,
                };

                let proceed = ask_permission(
                    &format!(
                        "Do you want to update task (#{}) - [{}]? (y/N)",
                        id, old_task.title
                    ),
                    force,
                )?;

                if proceed {
                    let task = edit_task(&mut conn, id, old_task, new_task)?;
                    println!("[TASK UPDATED] (#{}) - [{}]", id, task.title);
                } else {
                    println!("Operation Canceled")
                }
            }
            command::TaskCommandsEnum::List { status } => todo!(),
            command::TaskCommandsEnum::Read { id } => {
                let repository = TaskRepository::create(&conn);

                match repository.get_task(id)? {
                    Some(task) => {
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
                        println!("{}", "=".repeat(header.len()));
                    }
                    None => return Err(anyhow::anyhow!("Task with id (#{}) not found!", id).into()),
                }
            }
        },
        command::RootCommandsEnum::Undo { force } => {
            let repository = ActionRepository::create(&conn);

            let action = repository.get_last_unrestored_action()?;
            let proceed = ask_permission(&format!("Do you want to undo: {}? (y/N)", action.action.to_string()), force)?;
            if proceed {
                let result = undo_redo_operation(&mut conn, action)?;
                println!("{}", result)
            } else {
                println!("Operation Canceled")
            }
        }
        command::RootCommandsEnum::Redo { force } => {
            let repository = ActionRepository::create(&conn);

            let action = repository.get_fist_restored_action()?;
            let proceed = ask_permission(&format!("Do you want to redo: {}? (y/N)", action.action.to_string()), force)?;
            if proceed {
                let result = undo_redo_operation(&mut conn, action)?;
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
                println!("(#{}) - <{}> - [Restored: {}] - [{}]", action.id, action.action.to_string(), action.restored, action.created_at);
            }
        }
    }
    Ok(())
}

/**
 * Used to ask user for confirmation of action
 */
fn ask_permission(message: &str, force: bool) -> Result<bool> {
    if force {
        return Ok(true);
    }

    println!("{}", message);

    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;

    let trimmed_input = input.trim().to_lowercase();

    match trimmed_input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            return Err(
                anyhow::anyhow!("Invalid input. Please enter 'y', 'n', 'Y', or 'N'.").into(),
            )
        }
    }
}

fn get_date_format() -> &'static [time::format_description::BorrowedFormatItem<'static>] {
    format_description!("[year]-[month]-[day]")
}

fn parse_deadline(deadline: Option<String>) -> anyhow::Result<Option<Date>> {
    let format = get_date_format();

    deadline
        .map(|value| -> anyhow::Result<Date> { Ok(Date::parse(&value, &format)?) })
        .transpose()
}

fn parse_created_at(created_at: String) -> anyhow::Result<Date> {
    let format = get_date_format();

    match Date::parse(&created_at, &format) {
        Ok(created_at) => Ok(created_at),
        Err(_) => Err(anyhow::anyhow!("Invalid date: {}", created_at)),
    }
}

/**
 * Used to convert string "NOW" to Date struct
 */
fn parse_date_now(date: &str) -> String {
    use std::time::SystemTime;
    use time::OffsetDateTime;

    if date.eq("NOW") {
        // Get current time
        let now = SystemTime::now();
        let now = OffsetDateTime::from(now).date();
        // Format the date as YYYY-MM-DD
        let now = now.to_string();
        now
    } else {
        date.to_string()
    }
}
