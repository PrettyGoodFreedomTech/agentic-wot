use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub node_id: String,
    pub channels: Vec<ChannelInfo>,
    #[serde(default)]
    pub chain: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInfo {
    pub state: String,
    pub channel_id: String,
    pub balance_sat: i64,
    pub inbound_liquidity_sat: i64,
    #[serde(default)]
    pub capacity_sat: i64,
    #[serde(default)]
    pub funding_tx_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub balance_sat: i64,
    pub fee_credit_sat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub amount_sat: i64,
    pub payment_hash: String,
    pub serialized: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentResult {
    pub recipient_amount_sat: i64,
    pub routing_fee_sat: i64,
    pub payment_id: String,
    pub payment_hash: String,
    pub payment_preimage: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingPayment {
    pub payment_hash: String,
    pub preimage: String,
    #[serde(default)]
    pub external_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub invoice: String,
    pub is_paid: bool,
    pub received_sat: i64,
    pub fees: i64,
    pub completed_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingPayment {
    pub payment_id: String,
    pub payment_hash: String,
    pub preimage: Option<String>,
    pub is_paid: bool,
    pub sent: i64,
    pub fees: i64,
    #[serde(default)]
    pub invoice: String,
    pub completed_at: Option<i64>,
    pub created_at: i64,
}
