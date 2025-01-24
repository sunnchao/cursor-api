use serde::{Deserialize, Serialize};

use crate::{app::constant::COMMA, chat::constant::AVAILABLE_MODELS};

#[derive(Deserialize)]
pub struct BuildKeyRequest {
    // 认证令牌(必需)
    pub auth_token: String,
    // 流第一个块检查
    #[serde(default)]
    pub enable_stream_check: Option<bool>,
    // 包含停止流
    #[serde(default)]
    pub include_stop_stream: Option<bool>,
    // 是否禁用图片处理能力
    #[serde(default)]
    pub disable_vision: Option<bool>,
    // 慢速池
    #[serde(default)]
    pub enable_slow_pool: Option<bool>,
    // 使用量检查模型规则
    #[serde(default)]
    pub usage_check_models: Option<UsageCheckModelConfig>,
}
pub struct UsageCheckModelConfig {
    pub model_type: UsageCheckModelType,
    pub model_ids: Vec<&'static str>,
}

impl<'de> Deserialize<'de> for UsageCheckModelConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(rename = "type")]
            model_type: UsageCheckModelType,
            #[serde(default)]
            model_ids: String,
        }

        let helper = Helper::deserialize(deserializer)?;

        let model_ids = if helper.model_ids.is_empty() {
            Vec::new()
        } else {
            helper
                .model_ids
                .split(COMMA)
                .filter_map(|model| {
                    let model = model.trim();
                    AVAILABLE_MODELS
                        .iter()
                        .find(|m| m.id == model)
                        .map(|m| m.id)
                })
                .collect()
        };

        Ok(UsageCheckModelConfig {
            model_type: helper.model_type,
            model_ids,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UsageCheckModelType {
    Default,
    Disabled,
    All,
    Custom,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildKeyResponse {
    Key(String),
    Error(String),
}
