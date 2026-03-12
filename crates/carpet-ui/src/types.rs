#[derive(Debug, Clone, PartialEq)]
pub struct ListDisplay {
    pub coordinate: String,
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub item_count: usize,
    pub zap_count: u64,
    pub curator_name: String,
    pub curator_picture: Option<String>,
    pub curator_nip05: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BountyDisplay {
    pub d_tag: String,
    pub target_list_name: String,
    pub target_list_coordinate: String,
    pub reward_sats: u64,
    pub criteria: String,
    pub status: BountyStatus,
    pub creator_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BountyStatus {
    Open,
    Fulfilled,
    Expired,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemDisplay {
    pub resource: String,
    pub content: String,
    pub fields: Vec<(String, String)>,
}
