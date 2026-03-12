use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};

use crate::error::PhoenixdError;
use crate::types::{
    ChannelInfo, IncomingPayment, Invoice, NodeInfo, OutgoingPayment, PaymentResult, WalletBalance,
};

pub struct PhoenixdClient {
    http: Client,
    base_url: String,
    auth_header: HeaderValue,
}

impl PhoenixdClient {
    pub fn new(url: &str, password: &str) -> Self {
        use base64::Engine;
        let credentials = base64::engine::general_purpose::STANDARD.encode(format!(":{password}"));
        let auth_value =
            HeaderValue::from_str(&format!("Basic {credentials}")).expect("valid header");

        Self {
            http: Client::new(),
            base_url: url.trim_end_matches('/').to_string(),
            auth_header: auth_value,
        }
    }

    pub fn from_env() -> Result<Self, PhoenixdError> {
        let url = std::env::var("PHOENIXD_URL").unwrap_or_else(|_| "http://localhost:9740".into());
        let password = std::env::var("PHOENIXD_PASSWORD").map_err(|_| PhoenixdError::Api {
            message: "PHOENIXD_PASSWORD env var not set".into(),
        })?;
        Ok(Self::new(&url, &password))
    }

    pub async fn node_info(&self) -> Result<NodeInfo, PhoenixdError> {
        let resp = self
            .http
            .get(format!("{}/getinfo", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn get_balance(&self) -> Result<WalletBalance, PhoenixdError> {
        let resp = self
            .http
            .get(format!("{}/getbalance", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn create_invoice(
        &self,
        amount_sat: u64,
        description: &str,
        external_id: Option<&str>,
    ) -> Result<Invoice, PhoenixdError> {
        let mut params = vec![
            ("amountSat", amount_sat.to_string()),
            ("description", description.to_string()),
        ];
        if let Some(id) = external_id {
            params.push(("externalId", id.to_string()));
        }

        let resp = self
            .http
            .post(format!("{}/createinvoice", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .form(&params)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn pay_invoice(&self, bolt11: &str) -> Result<PaymentResult, PhoenixdError> {
        let resp = self
            .http
            .post(format!("{}/payinvoice", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .form(&[("invoice", bolt11)])
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn list_channels(&self) -> Result<Vec<ChannelInfo>, PhoenixdError> {
        let resp = self
            .http
            .get(format!("{}/listchannels", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn close_channel(
        &self,
        channel_id: &str,
        address: &str,
    ) -> Result<(), PhoenixdError> {
        let resp = self
            .http
            .delete(format!("{}/closechannel", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .form(&[("channelId", channel_id), ("address", address)])
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PhoenixdError::Api { message: text });
        }
        Ok(())
    }

    pub async fn get_incoming_payment(&self, hash: &str) -> Result<IncomingPayment, PhoenixdError> {
        let resp = self
            .http
            .get(format!("{}/payments/incoming/{hash}", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn get_outgoing_payment(&self, id: &str) -> Result<OutgoingPayment, PhoenixdError> {
        let resp = self
            .http
            .get(format!("{}/payments/outgoing/{id}", self.base_url))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, PhoenixdError> {
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PhoenixdError::Api { message: text });
        }
        let text = resp.text().await?;
        serde_json::from_str(&text).map_err(|e| PhoenixdError::Deserialize(format!("{e}: {text}")))
    }
}
