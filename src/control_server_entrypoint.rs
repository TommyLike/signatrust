#![allow(dead_code)]
use crate::util::error::Result;
use clap::Parser;
use config::Config;
use std::env;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

mod infra;
mod domain;
mod presentation;
mod util;
mod application;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

#[derive(Parser)]
#[command(name = "signatrust-control-server")]
#[command(author = "TommyLike <tommylikehu@gmail.com>")]
#[command(version = "0.10")]
#[command(about = "Signatrust control server for binary signing", long_about = None)]
pub struct App {
    #[arg(short, long)]
    #[arg(
    help = "path of configuration file, 'config/server.toml' relative to working directory be used in default"
    )]
    config: Option<String>,
}

lazy_static! {
    pub static ref SIGNAL: Arc<AtomicBool> = {
        let signal = Arc::new(AtomicBool::new(false));
        //setup up signal handler
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&signal)).expect("failed to register sigterm signal");
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&signal)).expect("failed to register sigint signal");
        signal
    };
    pub static ref SERVERCONFIG: Arc<RwLock<Config>> = {
        let app = App::parse();
        let path = app.config.unwrap_or(format!("{}/{}", env::current_dir().expect("current dir not found").display(),
            "config/server.toml"));
        let server_config = util::config::ServerConfig::new(path);
        server_config.watch(Arc::clone(&SIGNAL)).expect("failed to watch configure file");
        server_config.config
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    //prepare config and logger
    env_logger::init();
    //control server starts
    let control_server = presentation::server::control_server::ControlServer::new(SERVERCONFIG.clone()).await?;
    control_server.run().await?;
    Ok(())
}
