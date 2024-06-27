use anyhow::Result;
use clap::Parser;
use command::RootCommand;
use models::{setup_database, AddTask, UpdateTask};
use repositories::task_repository::TaskRepository;
use rusqlite::Connection;
use time::{macros::format_description, Date};

mod command;
mod models;
mod repositories;

fn main() -> Result<()> {
    let matches = RootCommand::parse();

    let conn = Connection::open(&matches.file)?;

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
                let format = format_description!("[year]-[month]-[day]");

                let deadline = deadline.map(|value| Date::parse(&value, &format).expect("Not reachable because variable verified before"));

                let created_at = Date::parse(&date, &format).expect("Not reachable because variable verified before");

                let task = AddTask {
                    title,
                    info,
                    deadline,
                    status,
                    created_at,
                };

                let repository = TaskRepository::create(&conn);

                let id = repository.add_task(task)?;

                println!("Task created with ID: {}", id);
            }
            command::TaskCommandsEnum::Delete { id, force } => todo!(),
            command::TaskCommandsEnum::Update {
                id,
                title,
                info,
                deadline,
                // categories,
                status,
                date,
                // force,
            } => {
                let format = format_description!("[year]-[month]-[day]");

                let info = info.map(|info| {
                    if info == "" {
                        None
                    } else {
                        Some(info)
                    }
                });

                let deadline = deadline.map(|deadline| {
                    if deadline == "" {
                        None
                    } else {
                        Some(Date::parse(&deadline, &format).expect("Not reachable because variable verified before"))
                    }
                });

                let created_at = date.map(|value| Date::parse(&value, &format).expect("Not reachable because variable verified before"));

                let task = UpdateTask {
                    id,
                    title,
                    info,
                    deadline,
                    status,
                    created_at,
                };

                let repository = TaskRepository::create(&conn);

                repository.edit_task(task)?;

                // TODO use category
                // TODO use force flag
            },
            command::TaskCommandsEnum::List { status } => todo!(),
            command::TaskCommandsEnum::Read { id } => todo!(),
        },
        command::RootCommandsEnum::Undo { force } => todo!(),
        command::RootCommandsEnum::Redo { force } => todo!(),
    }
    Ok(())
}
