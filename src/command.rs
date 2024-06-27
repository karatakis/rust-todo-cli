use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

use crate::models::TaskStatusEnum;

#[derive(Parser, Debug)]
#[command(version, about = "A CLI applicatio to manage TODO tasks", long_about = None)]
pub struct RootCommand {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "data.db",
        help = "Specifies the storage file",
        value_hint = ValueHint::FilePath,
    )]
    pub file: PathBuf,

    #[command(subcommand)]
    pub command: RootCommandsEnum,
}

#[derive(Subcommand, Debug)]
pub enum RootCommandsEnum {
    #[command(about = "All operations for tasks")]
    Task {
        #[command(subcommand)]
        command: TaskCommandsEnum,
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
        // #[arg(short, long, value_name = "CATEGORY", help = "Categories of the task")]
        // categories: Vec<String>,
        #[arg(long, short, value_name = "STATUS", help = "Status of the task", default_value=TaskStatusEnum::Undone)]
        status: TaskStatusEnum,
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
        // #[arg(short, long, value_name = "CATEGORY", help = "Categories of the task")]
        // categories: Vec<String>,
        #[arg(long, short, value_name = "STATUS", help = "Status of the task")]
        status: Option<TaskStatusEnum>,
        #[arg(
            long,
            short = 'a',
            value_name = "DATE",
            help = "Creation date of the task"
        )]
        date: Option<String>,
        // #[arg(short, long, help = "Force operation without confirmation")]
        // force: bool,
    },
    #[command(about = "List all the tasks based on query filters")]
    List {
        #[arg(long, short, value_name = "STATUS", help = "Status of the task", default_value=TaskStatusEnum::Undone)]
        status: TaskStatusEnum,
        /*
         * TODO:
         * Sorting category, date
         * Fuzzy search
         */
    },
    #[command(about = "Read an existing task")]
    Read {
        #[arg(index = 1, value_name = "ID", help = "The target task id")]
        id: i64,
    },
}
