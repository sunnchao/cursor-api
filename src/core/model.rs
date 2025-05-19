use std::sync::Arc;

use serde::{Deserialize, Serialize, ser::SerializeStruct as _};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Vision(Vec<VisionMessageContent>),
}

#[derive(Serialize, Deserialize)]
pub struct VisionMessageContent {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

#[derive(Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Copy,
    PartialEq,
)]
#[repr(u8)]
pub enum Role {
    #[serde(rename = "system", alias = "developer")]
    System = 0u8,
    #[serde(rename = "user", alias = "human")]
    User,
    #[serde(rename = "assistant", alias = "ai")]
    Assistant,
}

#[derive(Serialize)]
pub struct ChatResponse<'a> {
    pub id: &'a str,
    pub object: &'static str,
    pub created: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<&'static str>,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "TriState::is_none")]
    pub usage: TriState<Usage>,
}

#[derive(Serialize)]
pub struct Choice {
    pub index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<Delta>,
    pub logprobs: Option<bool>,
    pub finish_reason: Option<String>,
}

#[derive(Serialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

impl Default for Usage {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

// 聊天请求
#[derive(Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub stream_options: Option<StreamOptions>,
}

#[derive(Deserialize)]
pub struct StreamOptions {
    pub include_usage: bool,
}

/// 模型定义
#[derive(Clone, Copy)]
pub struct Model {
    pub id: &'static str,
    pub display_name: &'static str,
    pub created: &'static i64,
    pub object: &'static str,
    pub owned_by: &'static str,
    pub is_thinking: bool,
    pub is_image: bool,
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Model", 9)?;

        state.serialize_field("id", &self.id)?;
        state.serialize_field("display_name", &self.display_name)?;
        state.serialize_field("created", self.created)?;
        state.serialize_field("created_at", self.created)?;
        state.serialize_field("object", &self.object)?;
        state.serialize_field("type", &self.object)?;
        state.serialize_field("owned_by", &self.owned_by)?;
        state.serialize_field("supports_thinking", &self.is_thinking)?;
        state.serialize_field("supports_images", &self.is_image)?;

        state.end()
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

use super::constant::{FREE_MODELS, Models};
use crate::{
    app::model::{AppConfig, UsageCheck},
    common::model::tri::TriState,
};

impl Model {
    pub fn is_usage_check(&self, usage_check: Option<UsageCheck>) -> bool {
        match usage_check.unwrap_or(AppConfig::get_usage_check()) {
            UsageCheck::None => false,
            UsageCheck::Default => !FREE_MODELS.contains(&self.id),
            UsageCheck::All => true,
            UsageCheck::Custom(models) => models.contains(&self.id),
        }
    }
}

#[derive(Serialize)]
pub struct ModelsResponse {
    pub object: &'static str,
    pub data: Arc<Vec<Model>>,
}

impl ModelsResponse {
    #[inline]
    pub(super) fn new(data: Arc<Vec<Model>>) -> Self {
        Self {
            object: "list",
            data,
        }
    }

    #[inline]
    pub(super) fn with_default_models() -> Self {
        Self::new(Models::to_arc())
    }
}
