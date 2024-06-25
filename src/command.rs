use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A CLI applicatio to manage TODO tasks")]
pub struct RootCommand {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "data.db",
        help = "Specifies the storage file"
    )]
    file: PathBuf,

    #[command(subcommand)]
    command: Option<RootCommandsEnum>,
}

#[derive(Subcommand, Debug)]
pub enum RootCommandsEnum {
    #[command(about = "All operations for tasks")]
    Task {
        #[command(subcommand)]
        command: Option<TaskCommandsEnum>,
    },
    #[command(about = "Undo last operation")]
    Undo {
        #[arg(short, long, help = "Force operation without confirmation")]
        force: bool,
    },
    #[command(about = "Redo last undo operation")]
    Redo {
        #[arg(short, long, help = "Force operation without confirmation")]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommandsEnum {
    #[command(about = "Add a new task")]
    Add {
        #[arg(index = 1, value_name = "TITLE", help = "Title of the task")]
        title: String,
        #[arg(short, long, value_name = "INFO", help = "Info of the task")]
        info: Option<String>,
        #[arg(
            short,
            long,
            value_name = "DEADLINE",
            help = "Deadline date of the task"
        )]
        deadline: Option<String>,
        // TODO add categories
        #[arg(short, long, value_name = "CATEGORY", help = "Categories of the task")]
        categories: Vec<String>,
        #[arg(long, short, value_name = "STATUS", help = "Status of the task", default_value="undone", value_parser=["done", "undone", "completed"])]
        status: String,
        #[arg(
            long,
            short = 'a',
            value_name = "DATE",
            help = "Creation date of the task",
            default_value = "NOW"
        )]
        date: String,
    },
    #[command(about = "Delete an existing task")]
    Delete {
        #[arg(index = 1, value_name = "ID", help = "The target task id")]
        id: i64,
        #[arg(short, long, help = "Force operation without confirmation")]
        force: bool,
    },
    #[command(about = "Update an existing task")]
    Update {
        #[arg(index = 1, value_name = "ID", help = "The target task id")]
        id: i64,
        #[arg(long, short, value_name = "TITLE", help = "Title of the task")]
        title: Option<String>,
        #[arg(short, long, value_name = "INFO", help = "Info of the task")]
        info: Option<String>,
        #[arg(
            short,
            long,
            value_name = "DEADLINE",
            help = "Deadline date of the task"
        )]
        deadline: Option<String>,
        // TODO add categories
        #[arg(short, long, value_name = "CATEGORY", help = "Categories of the task")]
        categories: Vec<String>,
        #[arg(long, short, value_name = "STATUS", help = "Status of the task", value_parser=["done", "undone", "completed"])]
        status: Option<String>,
        #[arg(
            long,
            short = 'a',
            value_name = "DATE",
            help = "Creation date of the task",
            default_value = "NOW"
        )]
        date: Option<String>,
        #[arg(short, long, help = "Force operation without confirmation")]
        force: bool,
    },
    #[command(about = "List all the tasks based on query filters")]
    List {
        #[arg(long, short, value_name = "STATUS", help = "Status of the task", default_value="undone", value_parser=["done", "undone", "completed"])]
        status: String,
    },
    #[command(about = "Read an existing task")]
    Read {
        #[arg(index = 1, value_name = "ID", help = "The target task id")]
        id: i64,
    },
}
