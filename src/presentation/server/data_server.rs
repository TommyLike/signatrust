use std::net::SocketAddr;
use std::sync::{Arc, atomic::AtomicBool, RwLock};
use std::sync::atomic::Ordering;
use config::Config;
use tokio::fs;
use tokio::time::{Duration, sleep};
use tonic::{
    transport::{
        Certificate,
        Identity, Server, ServerTlsConfig,
    },
};
use crate::application::datakey::DBKeyService;

use crate::infra::database::model::datakey::repository;
use crate::infra::database::pool::{create_pool, get_db_pool};
use crate::infra::sign_backend::factory::SignBackendFactory;


use crate::presentation::handler::data::sign_handler::get_grpc_handler;
use crate::util::error::Result;

pub struct DataServer
{
    server_config: Arc<RwLock<Config>>,
    signal: Arc<AtomicBool>,
    server_identity: Option<Identity>,
    ca_cert: Option<Certificate>,
}

impl DataServer {
    pub async fn new(server_config: Arc<RwLock<Config>>, signal: Arc<AtomicBool>) -> Result<Self> {
        let database = server_config.read()?.get_table("database")?;
        create_pool(&database).await?;
        let mut server = DataServer {
            server_config,
            signal,
            server_identity: None,
            ca_cert: None,
        };
        server.load().await?;
        Ok(server)
    }

    async fn load(&mut self) -> Result<()> {
        if self
            .server_config
            .read()?
            .get_string("tls_cert")?
            .is_empty()
            || self
                .server_config
                .read()?
                .get_string("tls_key")?
                .is_empty()
        {
            info!("tls key and cert not configured, data server tls will be disabled");
            return Ok(());
        }
        self.ca_cert = Some(
            Certificate::from_pem(
                fs::read(self.server_config.read()?.get_string("ca_root")?).await?));
        self.server_identity = Some(Identity::from_pem(
            fs::read(self.server_config.read()?.get_string("tls_cert")?).await?,
            fs::read(self.server_config.read()?.get_string("tls_key")?).await?));
        Ok(())
    }

    async fn shutdown_signal(&self) {
        while !self.signal.load(Ordering::Relaxed) {
            sleep(Duration::from_secs(1)).await;
        }
        info!("quit signal received...")
    }

    pub async fn run(&self) -> Result<()> {
        //start grpc server
        let addr: SocketAddr = format!(
            "{}:{}",
            self.server_config
                .read()?
                .get_string("data-server.server_ip")?,
            self.server_config
                .read()?
                .get_string("data-server.server_port")?
        )
        .parse()?;

        let mut server = Server::builder();
        info!("data server starts");
        let sign_backend = SignBackendFactory::new_engine(
            self.server_config.clone(), get_db_pool()?).await?;
        let data_repository = repository::DataKeyRepository::new(
            get_db_pool()?);
        let key_service = DBKeyService::new(data_repository, sign_backend);
        if let Some(identity) = self.server_identity.clone() {
            server
                .tls_config(ServerTlsConfig::new().identity(identity).client_ca_root(self.ca_cert.clone().unwrap()))?
                .add_service(get_grpc_handler(key_service))
                .serve_with_shutdown(addr, self.shutdown_signal())
                .await?
        } else {
            server
                .add_service(get_grpc_handler(key_service))
                .serve_with_shutdown(addr, self.shutdown_signal())
                .await?
        }
        Ok(())
    }
}
