use anyhow::Result;
use clap::Parser;
use command::RootCommand;
use models::AddTask;
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

                let task = AddTask {
                    title,
                    info,
                    deadline: deadline.map(|value| Date::parse(&value, &format).expect("TODO")),
                    status,
                    created_at: Date::parse(&date, &format)?,
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
                categories,
                status,
                date,
                force,
            } => todo!(),
            command::TaskCommandsEnum::List { status } => todo!(),
            command::TaskCommandsEnum::Read { id } => todo!(),
        },
        command::RootCommandsEnum::Undo { force } => todo!(),
        command::RootCommandsEnum::Redo { force } => todo!(),
    }
    Ok(())
}

fn setup_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            info TEXT,
            deadline TEXT,
            status TEXT NOT NULL,
            created_at TEXT
        );
    ",
        (),
    )?;

    Ok(())
}
