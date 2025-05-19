mod header;
pub use header::*;

#[macro_export]
macro_rules! def_pub_const {
    // 单个常量定义
    // ($name:ident, $value:expr) => {
    //     pub const $name: &'static str = $value;
    // };

    // 批量常量定义
    ($($name:ident => $value:expr),+ $(,)?) => {
        $(
            pub const $name: &'static str = $value;
        )+
    };
}

pub const COMMA: char = ',';

// Package related constants
def_pub_const!(
    PKG_VERSION => env!("CARGO_PKG_VERSION")
    // PKG_NAME => env!("CARGO_PKG_NAME"),
    // PKG_DESCRIPTION => env!("CARGO_PKG_DESCRIPTION"),
    // PKG_AUTHORS => env!("CARGO_PKG_AUTHORS"),
    // PKG_REPOSITORY => env!("CARGO_PKG_REPOSITORY")
);

// Basic string constants
def_pub_const!(
    EMPTY_STRING => "",
    COMMA_STRING => ","
);

// Route related constants
def_pub_const!(
    ROUTE_ROOT_PATH => "/",
    ROUTE_HEALTH_PATH => "/health",
    ROUTE_GET_HASH => "/get-hash",
    ROUTE_GET_CHECKSUM => "/get-checksum",
    ROUTE_GET_TIMESTAMP_HEADER => "/get-tsheader",
    ROUTE_USER_INFO_PATH => "/userinfo",
    ROUTE_API_PATH => "/api",
    ROUTE_LOGS_PATH => "/logs",
    ROUTE_CONFIG_PATH => "/config",
    ROUTE_TOKENS_PATH => "/tokens",
    ROUTE_TOKENS_GET_PATH => "/tokens/get",
    ROUTE_TOKENS_SET_PATH => "/tokens/set",
    ROUTE_TOKENS_ADD_PATH => "/tokens/add",
    ROUTE_TOKENS_DELETE_PATH => "/tokens/del",
    ROUTE_TOKENS_TAGS_GET_PATH => "/tokens/tags/get",
    ROUTE_TOKENS_TAGS_SET_PATH => "/tokens/tags/set",
    ROUTE_TOKENS_BY_TAG_GET_PATH => "/tokens/by-tag/get",
    ROUTE_TOKENS_PROFILE_UPDATE_PATH => "/tokens/profile/update",
    ROUTE_TOKENS_UPGRADE_PATH => "/tokens/upgrade",
    ROUTE_TOKENS_STATUS_SET_PATH => "/tokens/status/set",
    ROUTE_PROXIES_PATH => "/proxies",
    ROUTE_PROXIES_GET_PATH => "/proxies/get",
    ROUTE_PROXIES_SET_PATH => "/proxies/set",
    ROUTE_PROXIES_ADD_PATH => "/proxies/add",
    ROUTE_PROXIES_DELETE_PATH => "/proxies/del",
    ROUTE_PROXIES_SET_GENERAL_PATH => "/proxies/set-general",
    ROUTE_ENV_EXAMPLE_PATH => "/env-example",
    ROUTE_STATIC_PATH => "/static/{path}",
    ROUTE_SHARED_STYLES_PATH => "/static/shared-styles.css",
    ROUTE_SHARED_JS_PATH => "/static/shared.js",
    ROUTE_ABOUT_PATH => "/about",
    ROUTE_README_PATH => "/readme",
    ROUTE_BASIC_CALIBRATION_PATH => "/basic-calibration",
    ROUTE_BUILD_KEY_PATH => "/build-key",
    ROUTE_TOKEN_UPGRADE_PATH => "/token-upgrade"
);

// def_pub_const!(DEFAULT_TOKEN_LIST_FILE_NAME => ".tokens");

// Status constants
def_pub_const!(
    STATUS_PENDING => "pending",
    STATUS_SUCCESS => "success",
    STATUS_FAILURE => "failure"
);

// Boolean constants
def_pub_const!(
    TRUE => "true",
    FALSE => "false"
);

// Authorization constants
def_pub_const!(
    AUTHORIZATION_BEARER_PREFIX => "Bearer "
);

// Cursor related constants
def_pub_const!(
    CURSOR_API2_HOST => "api2.cursor.sh",
    CURSOR_HOST => "www.cursor.com",
    CURSOR_SETTINGS_URL => "https://www.cursor.com/settings"
);

// Object type constants
def_pub_const!(
    OBJECT_CHAT_COMPLETION => "chat.completion",
    OBJECT_CHAT_COMPLETION_CHUNK => "chat.completion.chunk",
    // OBJECT_TEXT_COMPLETION => "text_completion"
);

// def_pub_const!(
//     CURSOR_API2_STREAM_CHAT => "StreamChat",
//     CURSOR_API2_GET_USER_INFO => "GetUserInfo"
// );

// Finish reason constants
def_pub_const!(
    FINISH_REASON_STOP => "stop"
);

// Error message constants
def_pub_const!(
    ERR_INVALID_PATH => "无效的路径"
);

// def_pub_const!(ERR_CHECKSUM_NO_GOOD => "checksum no good");

// Claude system prompts
def_pub_const!(
    SYSTEM_PROMPT_CLAUDE_3_7_SONNET_20250224 => include_str!("prompts/Claude 3.7 Sonnet"),
    SYSTEM_PROMPT_CLAUDE_3_5_SONNET_20241122_TEXT_ONLY => include_str!("prompts/Claude 3.5 Sonnet Text only"),
    SYSTEM_PROMPT_CLAUDE_3_5_SONNET_20241122_TEXT_AND_IMAGES => include_str!("prompts/Claude 3.5 Sonnet Text and images"),
    SYSTEM_PROMPT_CLAUDE_3_OPUS_20240712 => include_str!("prompts/Claude 3 Opus"),
    SYSTEM_PROMPT_CLAUDE_3_HAIKU_20240712 => include_str!("prompts/Claude 3 Haiku")
);
