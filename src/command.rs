use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use time::Date;

use crate::{
    models::TaskStatusEnum,
    utils::{created_at_parser, date_parser},
};

/**
 * The CLI parser of arguments
 */
#[derive(Parser, Debug)]
#[command(version, about = "A CLI application to manage TODO tasks", long_about = None)]
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
    #[command(about = "List last actions")]
    Actions {
        #[arg(short, long, help = "Number of items to show", default_value = "10", value_parser = clap::value_parser!(u64).range(1..))]
        limit: u64,
    },
    #[command(about = "All operations for task categories")]
    Category {
        #[command(subcommand)]
        command: CategoryCommandsEnum,
    },
}

#[derive(Subcommand, Debug)]
pub enum CategoryCommandsEnum {
    #[command(about = "List all categories")]
    List,
    #[command(about = "Add a category to a task")]
    Add {
        #[arg(index = 1, value_name = "TASK ID", help = "ID of the task", value_parser = clap::value_parser!(i64).range(1..))]
        task_id: i64,
        #[arg(index = 2, value_name = "CATEGORY", help = "The task category")]
        category: String,
    },
    #[command(about = "Rename a category from a task")]
    Rename {
        #[arg(index = 1, value_name = "TASK ID", help = "ID of the task", value_parser = clap::value_parser!(i64).range(1..))]
        task_id: i64,
        #[arg(index = 2, value_name = "OLD CATEGORY", help = "The old task category")]
        old_category: String,
        #[arg(index = 3, value_name = "NEW CATEGORY", help = "The new task category")]
        new_category: String,
    },
    #[command(about = "Remove a category from a task")]
    Remove {
        #[arg(index = 1, value_name = "TASK ID", help = "ID of the task", value_parser = clap::value_parser!(i64).range(1..))]
        task_id: i64,
        #[arg(index = 2, value_name = "CATEGORY", help = "The task category")]
        category: String,
    },
    #[command(about = "Batch rename a task")]
    BatchRename {
        #[arg(index = 1, value_name = "OLD CATEGORY", help = "The old task category")]
        old_category: String,
        #[arg(index = 2, value_name = "NEW CATEGORY", help = "The new task category")]
        new_category: String,
    },
    #[command(about = "Batch delete a task")]
    BatchDelete {
        #[arg(index = 1, value_name = "CATEGORY", help = "The task category")]
        category: String,
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
            help = "Deadline date of the task",
            value_parser = date_parser
        )]
        deadline: Option<Date>,
        #[arg(short, long, value_name = "CATEGORY", help = "Categories of the task")]
        categories: Option<Vec<String>>,
        #[arg(long, short, value_name = "STATUS", help = "Status of the task", default_value=TaskStatusEnum::Undone)]
        status: TaskStatusEnum,
        #[arg(
            long,
            short = 'a',
            value_name = "DATE",
            help = "Creation date of the task",
            default_value = "NOW",
            value_parser = created_at_parser
        )]
        date: Date,
    },
    #[command(about = "Delete an existing task")]
    Delete {
        #[arg(index = 1, value_name = "ID", help = "The target task id", value_parser = clap::value_parser!(i64).range(1..))]
        id: i64,
        #[arg(short, long, help = "Force operation without confirmation")]
        force: bool,
    },
    #[command(about = "Update an existing task")]
    Update {
        #[arg(index = 1, value_name = "ID", help = "The target task id", value_parser = clap::value_parser!(i64).range(1..))]
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
        #[arg(long, short, value_name = "STATUS", help = "Status of the task")]
        status: Option<TaskStatusEnum>,
        #[arg(
            long,
            short = 'a',
            value_name = "DATE",
            help = "Creation date of the task",
            value_parser = created_at_parser
        )]
        date: Option<Date>,
        #[arg(short, long, help = "Force operation without confirmation")]
        force: bool,
    },
    #[command(about = "List all the tasks based on query filters")]
    List {
        #[arg(long, short, value_name = "STATUS", help = "Status of the task")]
        status: Option<TaskStatusEnum>,
        /*
         * TODO:
         * Sorting category, date
         * Fuzzy search
         */
    },
    #[command(about = "Read an existing task")]
    Read {
        #[arg(index = 1, value_name = "ID", help = "The target task id", value_parser = clap::value_parser!(i64).range(1..))]
        id: i64,
    },
}
