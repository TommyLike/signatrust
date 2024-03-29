#![allow(dead_code)]
use std::env;
use crate::util::error::{Result, Error};
use clap::{Parser, Subcommand};
use crate::client::cmd::add;
use config::{Config, File};
use std::sync::{Arc, atomic::AtomicBool, RwLock};
use crate::client::cmd::traits::SignCommand;

mod util;
mod client;
mod infra;
mod domain;
mod application;
mod presentation;


#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

#[derive(Parser)]
#[command(name = "signatrust-client")]
#[command(author = "TommyLike <tommylikehu@gmail.com>")]
#[command(version = "0.10")]
#[command(about = "Sign binary with specified key id", long_about = None)]
pub struct App {
    #[arg(short, long)]
    #[arg(
    help = "path of configuration file, './client.toml' relative to working directory be used in default"
    )]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create new signature for single file or all of the files in directory", long_about = None)]
    Add(add::CommandAdd),
}

fn main() -> Result<()> {
    //prepare config and logger
    env_logger::init();
    let app = App::parse();
    let path = app.config.unwrap_or(
        format!("{}/{}", env::current_dir().expect("current dir not found").display(), "client.toml"));
    let mut client = Config::default();
    client
        .merge(File::with_name(path.as_str())).expect("load client configuration file");
    let signal = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&signal)).expect("failed to register sigterm signal");
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&signal)).expect("failed to register sigint signal");
    //construct handler
    let command = match app.command {
        Some(Commands::Add(add_command)) => {
            Some(add::CommandAddHandler::new(signal.clone(), Arc::new(RwLock::new(client)), add_command)?)
        }
        None => {None}
    };
    //handler and quit
    if let Some(handler) = command {
        handler.validate().expect("failed to validate command option");
        if !handler.handle().expect("failed to perform command") {
            return Err(Error::PartialFailureError)
        }
    }
    Ok(())
}
