use serde::{Deserialize, Serialize};

use crate::app::model::{PageContent, UsageCheck, VisionAbility};

#[derive(Serialize)]
pub struct ConfigData {
    pub page_content: Option<PageContent>,
    pub vision_ability: VisionAbility,
    pub enable_slow_pool: bool,
    pub enable_long_context: bool,
    pub usage_check_models: UsageCheck,
    pub enable_dynamic_key: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub share_token: String,
    pub include_web_references: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ConfigUpdateRequest {
    pub action: String, // "get", "update", "reset"
    pub path: String,
    pub content: Option<PageContent>, // "default", "text", "html"
    pub vision_ability: Option<VisionAbility>,
    pub enable_slow_pool: Option<bool>,
    pub enable_long_context: Option<bool>,
    pub usage_check_models: Option<UsageCheck>,
    pub enable_dynamic_key: Option<bool>,
    pub share_token: Option<String>,
    pub include_web_references: Option<bool>,
}
