/// Marketplace view state tracked in the UI.
#[derive(Debug, Clone, Default)]
pub struct MarketplaceState {
    pub search_query: String,
    pub selected_category: Option<String>,
    pub is_loading: bool,
}
