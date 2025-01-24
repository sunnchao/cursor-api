use crate::{
    app::constant::{
        EMPTY_STRING, ERR_INVALID_PATH, ROUTE_ABOUT_PATH, ROUTE_API_PATH, ROUTE_BUILD_KEY_PATH,
        ROUTE_CONFIG_PATH, ROUTE_LOGS_PATH, ROUTE_README_PATH, ROUTE_ROOT_PATH,
        ROUTE_SHARED_JS_PATH, ROUTE_SHARED_STYLES_PATH, ROUTE_TOKENS_PATH,
    },
    chat::model::Message,
    common::{
        client::rebuild_http_client,
        model::{userinfo::TokenProfile, ApiStatus},
        utils::{generate_checksum_with_repair, parse_bool_from_env, parse_string_from_env},
    },
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

mod usage_check;
pub use usage_check::UsageCheck;
mod proxies;
pub use proxies::Proxies;
mod build_key;
pub use build_key::*;

// 页面内容类型枚举
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PageContent {
    #[serde(rename = "default")]
    Default, // 默认行为
    #[serde(rename = "text")]
    Text(String), // 纯文本
    #[serde(rename = "html")]
    Html(String), // HTML 内容
}

impl Default for PageContent {
    fn default() -> Self {
        Self::Default
    }
}

// 静态配置
#[derive(Clone)]
pub struct AppConfig {
    stream_check: bool,
    stop_stream: bool,
    vision_ability: VisionAbility,
    slow_pool: bool,
    allow_claude: bool,
    pages: Pages,
    usage_check: UsageCheck,
    dynamic_key: bool,
    share_token: String,
    is_share: bool,
    proxies: Proxies,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum VisionAbility {
    #[serde(rename = "none", alias = "disabled")]
    None,
    #[serde(rename = "base64", alias = "base64-only")]
    Base64,
    #[serde(rename = "all", alias = "base64-http")]
    All,
}

impl VisionAbility {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" | "disabled" => Self::None,
            "base64" | "base64-only" => Self::Base64,
            "all" | "base64-http" => Self::All,
            _ => Self::default(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, VisionAbility::None)
    }
}

impl Default for VisionAbility {
    fn default() -> Self {
        Self::Base64
    }
}

#[derive(Clone, Default)]
pub struct Pages {
    pub root_content: PageContent,
    pub logs_content: PageContent,
    pub config_content: PageContent,
    pub tokeninfo_content: PageContent,
    pub shared_styles_content: PageContent,
    pub shared_js_content: PageContent,
    pub about_content: PageContent,
    pub readme_content: PageContent,
    pub api_content: PageContent,
    pub build_key_content: PageContent,
}

// 运行时状态
pub struct AppState {
    pub total_requests: u64,
    pub active_requests: u64,
    pub error_requests: u64,
    pub request_logs: Vec<RequestLog>,
    pub token_infos: Vec<TokenInfo>,
}

// 全局配置实例
pub static APP_CONFIG: LazyLock<RwLock<AppConfig>> =
    LazyLock::new(|| RwLock::new(AppConfig::default()));

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            stream_check: true,
            stop_stream: true,
            vision_ability: VisionAbility::Base64,
            slow_pool: false,
            allow_claude: false,
            pages: Pages::default(),
            usage_check: UsageCheck::Default,
            dynamic_key: false,
            share_token: String::default(),
            is_share: false,
            proxies: Proxies::default(),
        }
    }
}

macro_rules! config_methods {
    ($($field:ident: $type:ty, $default:expr;)*) => {
        $(
            paste::paste! {
                pub fn [<get_ $field>]() -> $type
                where
                    $type: Copy + PartialEq,
                {
                    APP_CONFIG.read().$field
                }

                pub fn [<update_ $field>](value: $type)
                where
                    $type: Copy + PartialEq,
                {
                    let current = Self::[<get_ $field>]();
                    if current != value {
                        APP_CONFIG.write().$field = value;
                    }
                }

                pub fn [<reset_ $field>]()
                where
                    $type: Copy + PartialEq,
                {
                    let default_value = $default;
                    let current = Self::[<get_ $field>]();
                    if current != default_value {
                        APP_CONFIG.write().$field = default_value;
                    }
                }
            }
        )*
    };
}

macro_rules! config_methods_clone {
    ($($field:ident: $type:ty, $default:expr;)*) => {
        $(
            paste::paste! {
                pub fn [<get_ $field>]() -> $type
                where
                    $type: Clone + PartialEq,
                {
                    APP_CONFIG.read().$field.clone()
                }

                pub fn [<update_ $field>](value: $type)
                where
                    $type: Clone + PartialEq,
                {
                    let current = Self::[<get_ $field>]();
                    if current != value {
                        APP_CONFIG.write().$field = value;
                    }
                }

                pub fn [<reset_ $field>]()
                where
                    $type: Clone + PartialEq,
                {
                    let default_value = $default;
                    let current = Self::[<get_ $field>]();
                    if current != default_value {
                        APP_CONFIG.write().$field = default_value;
                    }
                }
            }
        )*
    };
}

impl AppConfig {
    pub fn init() {
        let mut config = APP_CONFIG.write();
        config.stream_check = parse_bool_from_env("ENABLE_STREAM_CHECK", true);
        config.stop_stream = parse_bool_from_env("INCLUDE_STOP_REASON_STREAM", true);
        config.vision_ability =
            VisionAbility::from_str(&parse_string_from_env("VISION_ABILITY", EMPTY_STRING));
        config.slow_pool = parse_bool_from_env("ENABLE_SLOW_POOL", false);
        config.allow_claude = parse_bool_from_env("PASS_ANY_CLAUDE", false);
        config.usage_check =
            UsageCheck::from_str(&parse_string_from_env("USAGE_CHECK", EMPTY_STRING));
        config.dynamic_key = parse_bool_from_env("DYNAMIC_KEY", false);
        config.share_token = parse_string_from_env("SHARED_TOKEN", EMPTY_STRING);
        config.is_share = !config.share_token.is_empty();
        config.proxies = match std::env::var("PROXIES") {
            Ok(proxies) => Proxies::from_str(proxies.as_str()),
            Err(_) => Proxies::default(),
        }
    }

    config_methods! {
        stream_check: bool, true;
        stop_stream: bool, true;
        slow_pool: bool, false;
        allow_claude: bool, false;
        dynamic_key: bool, false;
    }

    config_methods_clone! {
        vision_ability: VisionAbility, VisionAbility::default();
        usage_check: UsageCheck, UsageCheck::default();
    }

    pub fn get_share_token() -> String {
        APP_CONFIG.read().share_token.clone()
    }

    pub fn update_share_token(value: String) {
        let current = Self::get_share_token();
        if current != value {
            let mut config = APP_CONFIG.write();
            config.share_token = value;
            config.is_share = !config.share_token.is_empty();
        }
    }

    pub fn reset_share_token() {
        let current = Self::get_share_token();
        if !current.is_empty() {
            let mut config = APP_CONFIG.write();
            config.share_token = String::new();
            config.is_share = false;
        }
    }

    pub fn get_proxies() -> Proxies {
        APP_CONFIG.read().proxies.clone()
    }

    pub fn update_proxies(value: Proxies) {
        let current = Self::get_proxies();
        if current != value {
            let mut config = APP_CONFIG.write();
            config.proxies = value;
            rebuild_http_client();
        }
    }

    pub fn reset_proxies() {
        let default_value = Proxies::default();
        let current = Self::get_proxies();
        if current != default_value {
            let mut config = APP_CONFIG.write();
            config.proxies = default_value;
            rebuild_http_client();
        }
    }

    pub fn get_page_content(path: &str) -> Option<PageContent> {
        match path {
            ROUTE_ROOT_PATH => Some(APP_CONFIG.read().pages.root_content.clone()),
            ROUTE_LOGS_PATH => Some(APP_CONFIG.read().pages.logs_content.clone()),
            ROUTE_CONFIG_PATH => Some(APP_CONFIG.read().pages.config_content.clone()),
            ROUTE_TOKENS_PATH => Some(APP_CONFIG.read().pages.tokeninfo_content.clone()),
            ROUTE_SHARED_STYLES_PATH => Some(APP_CONFIG.read().pages.shared_styles_content.clone()),
            ROUTE_SHARED_JS_PATH => Some(APP_CONFIG.read().pages.shared_js_content.clone()),
            ROUTE_ABOUT_PATH => Some(APP_CONFIG.read().pages.about_content.clone()),
            ROUTE_README_PATH => Some(APP_CONFIG.read().pages.readme_content.clone()),
            ROUTE_API_PATH => Some(APP_CONFIG.read().pages.api_content.clone()),
            ROUTE_BUILD_KEY_PATH => Some(APP_CONFIG.read().pages.build_key_content.clone()),
            _ => None,
        }
    }

    pub fn update_page_content(path: &str, content: PageContent) -> Result<(), &'static str> {
        let mut config = APP_CONFIG.write();
        match path {
            ROUTE_ROOT_PATH => config.pages.root_content = content,
            ROUTE_LOGS_PATH => config.pages.logs_content = content,
            ROUTE_CONFIG_PATH => config.pages.config_content = content,
            ROUTE_TOKENS_PATH => config.pages.tokeninfo_content = content,
            ROUTE_SHARED_STYLES_PATH => config.pages.shared_styles_content = content,
            ROUTE_SHARED_JS_PATH => config.pages.shared_js_content = content,
            ROUTE_ABOUT_PATH => config.pages.about_content = content,
            ROUTE_README_PATH => config.pages.readme_content = content,
            ROUTE_API_PATH => config.pages.api_content = content,
            ROUTE_BUILD_KEY_PATH => config.pages.build_key_content = content,
            _ => return Err(ERR_INVALID_PATH),
        }
        Ok(())
    }

    pub fn reset_page_content(path: &str) -> Result<(), &'static str> {
        let mut config = APP_CONFIG.write();
        match path {
            ROUTE_ROOT_PATH => config.pages.root_content = PageContent::default(),
            ROUTE_LOGS_PATH => config.pages.logs_content = PageContent::default(),
            ROUTE_CONFIG_PATH => config.pages.config_content = PageContent::default(),
            ROUTE_TOKENS_PATH => config.pages.tokeninfo_content = PageContent::default(),
            ROUTE_SHARED_STYLES_PATH => config.pages.shared_styles_content = PageContent::default(),
            ROUTE_SHARED_JS_PATH => config.pages.shared_js_content = PageContent::default(),
            ROUTE_ABOUT_PATH => config.pages.about_content = PageContent::default(),
            ROUTE_README_PATH => config.pages.readme_content = PageContent::default(),
            ROUTE_API_PATH => config.pages.api_content = PageContent::default(),
            ROUTE_BUILD_KEY_PATH => config.pages.build_key_content = PageContent::default(),
            _ => return Err(ERR_INVALID_PATH),
        }
        Ok(())
    }

    pub fn is_share() -> bool {
        APP_CONFIG.read().is_share
    }
}

impl AppState {
    pub fn new(token_infos: Vec<TokenInfo>) -> Self {
        Self {
            total_requests: 0,
            active_requests: 0,
            error_requests: 0,
            request_logs: Vec::new(),
            token_infos,
        }
    }

    pub fn update_checksum(&mut self) {
        for token_info in self.token_infos.iter_mut() {
            token_info.checksum = generate_checksum_with_repair(&token_info.checksum);
        }
    }
}

// 请求日志
#[derive(Serialize, Clone)]
pub struct RequestLog {
    pub id: u64,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub model: String,
    pub token_info: TokenInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    pub timing: TimingInfo,
    pub stream: bool,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TimingInfo {
    pub total: f64, // 总用时(秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<f64>, // 首字时间(秒)
}

// 聊天请求
#[derive(Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
}

// 用于存储 token 信息
#[derive(Serialize, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub checksum: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<TokenProfile>,
}

// TokenUpdateRequest 结构体
#[derive(Deserialize)]
pub struct TokenUpdateRequest {
    pub tokens: String,
}

#[derive(Deserialize)]
pub struct TokenAddRequestTokenInfo {
    pub token: String,
    #[serde(default)]
    pub checksum: Option<String>,
}

// TokensDeleteRequest 结构体
#[derive(Deserialize)]
pub struct TokensDeleteRequest {
    #[serde(default)]
    pub tokens: Vec<String>,
    #[serde(default)]
    pub expectation: TokensDeleteResponseExpectation,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TokensDeleteResponseExpectation {
    #[default]
    Simple,
    UpdatedTokens,
    FailedTokens,
    Detailed,
}

impl TokensDeleteResponseExpectation {
    pub fn needs_updated_tokens(&self) -> bool {
        matches!(
            self,
            TokensDeleteResponseExpectation::UpdatedTokens
                | TokensDeleteResponseExpectation::Detailed
        )
    }

    pub fn needs_failed_tokens(&self) -> bool {
        matches!(
            self,
            TokensDeleteResponseExpectation::FailedTokens
                | TokensDeleteResponseExpectation::Detailed
        )
    }
}

// TokensDeleteResponse 结构体
#[derive(Serialize)]
pub struct TokensDeleteResponse {
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_tokens: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_tokens: Option<Vec<String>>,
}
