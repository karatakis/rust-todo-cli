use clap::Parser;
use command::RootCommand;

pub mod command;
pub mod models;

fn main() {
    let matches = RootCommand::parse();

    println!("{:?}", matches);
}
