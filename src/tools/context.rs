use crate::api::CozeApiClient;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub coze_client: Arc<CozeApiClient>,
    pub user_id: String,
    pub session_id: String,
    pub workspace_id: Option<String>,
    pub metadata: HashMap<String, String>,
}
