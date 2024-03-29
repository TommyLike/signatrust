use tonic::transport::{Channel, ClientTlsConfig, Identity};
use std::collections::HashMap;
use config::Value;
use crate::client::load_balancer::dns::DNSLoadBalancer;
use crate::client::load_balancer::single::SingleLoadBalancer;
use crate::client::load_balancer::traits::DynamicLoadBalancer;
use crate::util::error::{Error, Result};

pub struct ChannelFactory {
    lb: Box<dyn DynamicLoadBalancer>
}

impl ChannelFactory {
    pub async fn new(config: &HashMap<String, Value>) -> Result<Self> {
        let mut client_config :Option<ClientTlsConfig> = None;
        let tls_cert = config.get("tls_cert").unwrap_or(&Value::default()).to_string();
        let tls_key = config.get("tls_key").unwrap_or(&Value::default()).to_string();
        let server_port = config.get("server_port").unwrap_or(&Value::default()).to_string();
        if tls_cert.is_empty() || tls_key.is_empty()
        {
            info!("tls client key and cert not configured, tls will be disabled");
        } else {
            info!("tls client key and cert configured, tls will be enabled");
            let identity = Identity::from_pem(
                tokio::fs::read(tls_cert).await?,
                tokio::fs::read(tls_key).await?);
            client_config = Some(ClientTlsConfig::new()
                .identity(identity).domain_name(config.get("domain_name").unwrap_or(&Value::default()).to_string()));
        }
        let lb_type = config.get("type").unwrap_or(&Value::default()).to_string();
        if lb_type == "single" {
            return Ok(Self {
                lb: Box::new(SingleLoadBalancer::new(
                    config.get("server_address").unwrap_or(&Value::default()).to_string(),
                    server_port, client_config)?)
            })
        } else if lb_type == "dns" {
            return Ok(Self {
                lb: Box::new(DNSLoadBalancer::new(
                    config.get("server_name").unwrap_or(&Value::default()).to_string(),
                    server_port, client_config)?)
            })
        }
        Err(Error::ConfigError(format!("invalid load balancer type configuration {}", lb_type)))
    }

    pub fn get_channel(&self) -> Result<Channel> {
        self.lb.get_transport_channel()
    }
}