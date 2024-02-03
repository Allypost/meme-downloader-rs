use std::time::Duration;

use reqwest::blocking::{Client as ReqwestClient, ClientBuilder as ReqwestClientBuilder};

use super::USER_AGENT;

pub struct Client;

impl Client {
    pub fn default() -> Result<ReqwestClient, String> {
        Self::builder()
            .build()
            .map_err(|e| format!("Failed to create client: {:?}", e))
    }

    pub fn builder() -> ReqwestClientBuilder {
        ReqwestClient::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(5))
    }
}
