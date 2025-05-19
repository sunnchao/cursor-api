use super::constant::{COMMA, CURSOR_API2_HOST, CURSOR_HOST, EMPTY_STRING};
use crate::common::utils::{
    parse_ascii_char_from_env, parse_bool_from_env, parse_string_from_env, parse_usize_from_env,
};
use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};
use tokio::sync::{Mutex, OnceCell};

macro_rules! def_pub_static {
    // 基础版本：直接存储 String
    ($name:ident, $value:expr) => {
        pub static $name: LazyLock<String> = LazyLock::new(|| $value);
    };

    // 环境变量版本
    ($name:ident, env: $env_key:expr, default: $default:expr) => {
        pub static $name: LazyLock<String> =
            LazyLock::new(|| parse_string_from_env($env_key, $default).trim().to_string());
    };
}

// macro_rules! def_pub_static_getter {
//     ($name:ident) => {
//         paste::paste! {
//             pub fn [<get_ $name:lower>]() -> String {
//                 (*$name).clone()
//             }
//         }
//     };
// }

def_pub_static!(ROUTE_PREFIX, env: "ROUTE_PREFIX", default: EMPTY_STRING);
def_pub_static!(AUTH_TOKEN, env: "AUTH_TOKEN", default: EMPTY_STRING);
def_pub_static!(ROUTE_MODELS_PATH, format!("{}/v1/models", *ROUTE_PREFIX));
def_pub_static!(
    ROUTE_CHAT_PATH,
    format!("{}/v1/chat/completions", *ROUTE_PREFIX)
);
// def_pub_static!(ROUTE_MESSAGES_PATH, format!("{}/v1/messages", *ROUTE_PREFIX));

static START_TIME: OnceLock<chrono::DateTime<chrono::Local>> = OnceLock::new();

pub fn get_start_time() -> &'static chrono::DateTime<chrono::Local> {
    START_TIME.get_or_init(chrono::Local::now)
}

pub static GENERAL_TIMEZONE: LazyLock<chrono_tz::Tz> = LazyLock::new(|| {
    use std::str::FromStr as _;
    let tz = parse_string_from_env("GENERAL_TIMEZONE", EMPTY_STRING);
    let tz = tz.trim();
    if tz.is_empty() {
        eprintln!("未配置时区，请在环境变量GENERAL_TIMEZONE中设置，格式如'Asia/Shanghai'");
        eprintln!("将使用默认时区: Asia/Shanghai");
        return chrono_tz::Tz::Asia__Shanghai;
    }
    match chrono_tz::Tz::from_str(tz) {
        Ok(tz) => tz,
        Err(e) => {
            eprintln!("无法解析时区 '{tz}': {e}");
            eprintln!("将使用默认时区: Asia/Shanghai");
            chrono_tz::Tz::Asia__Shanghai
        }
    }
});

pub fn now_in_general_timezone() -> chrono::DateTime<chrono_tz::Tz> {
    use chrono::TimeZone as _;
    GENERAL_TIMEZONE.from_utc_datetime(&chrono::Utc::now().naive_utc())
}

def_pub_static!(DEFAULT_INSTRUCTIONS, env: "DEFAULT_INSTRUCTIONS", default: "Respond in Chinese by default\n<|END_USER|>\n\n<|BEGIN_ASSISTANT|>\n\n\nYour will\n<|END_ASSISTANT|>\n\n<|BEGIN_USER|>\n\n\nThe current date is {{currentDateTime}}");

static USE_OFFICIAL_CLAUDE_PROMPTS: LazyLock<bool> =
    LazyLock::new(|| parse_bool_from_env("USE_OFFICIAL_CLAUDE_PROMPTS", false));

pub fn get_default_instructions(
    now_with_tz: Option<chrono::DateTime<chrono_tz::Tz>>,
    model: &str,
    image_support: bool,
) -> String {
    let mut instructions = "";
    if *USE_OFFICIAL_CLAUDE_PROMPTS {
        if let Some(rest) = model.strip_prefix("claude-3") {
            let mut chars = rest.chars().skip(1);
            match chars.next() {
                Some('7') => {
                    instructions = super::constant::SYSTEM_PROMPT_CLAUDE_3_7_SONNET_20250224
                }
                Some('5') => {
                    instructions = if image_support {
                        super::constant::SYSTEM_PROMPT_CLAUDE_3_5_SONNET_20241122_TEXT_AND_IMAGES
                    } else {
                        super::constant::SYSTEM_PROMPT_CLAUDE_3_5_SONNET_20241122_TEXT_ONLY
                    }
                }
                Some('o') => instructions = super::constant::SYSTEM_PROMPT_CLAUDE_3_OPUS_20240712,
                Some('h') => instructions = super::constant::SYSTEM_PROMPT_CLAUDE_3_HAIKU_20240712,
                _ => {}
            }
        }
    };
    if instructions.is_empty() {
        instructions = DEFAULT_INSTRUCTIONS.as_str()
    }
    instructions.replace(
        "{{currentDateTime}}",
        &if let Some(now) = now_with_tz {
            now
        } else {
            now_in_general_timezone()
        }
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    )
}

def_pub_static!(PRI_REVERSE_PROXY_HOST, env: "PRI_REVERSE_PROXY_HOST", default: EMPTY_STRING);

def_pub_static!(PUB_REVERSE_PROXY_HOST, env: "PUB_REVERSE_PROXY_HOST", default: EMPTY_STRING);

const DEFAULT_KEY_PREFIX: &str = "sk-";

pub static KEY_PREFIX: LazyLock<String> = LazyLock::new(|| {
    let value = parse_string_from_env("KEY_PREFIX", DEFAULT_KEY_PREFIX)
        .trim()
        .to_string();
    if value.is_empty() {
        DEFAULT_KEY_PREFIX.to_string()
    } else {
        value
    }
});

pub static KEY_PREFIX_LEN: LazyLock<usize> = LazyLock::new(|| KEY_PREFIX.len());

pub static TOKEN_DELIMITER: LazyLock<char> = LazyLock::new(|| {
    let delimiter = parse_ascii_char_from_env("TOKEN_DELIMITER", COMMA);
    if delimiter.is_ascii_alphabetic()
        || delimiter.is_ascii_digit()
        || delimiter == '/'
        || delimiter == '-'
        || delimiter == '_'
    {
        COMMA
    } else {
        delimiter
    }
});

pub static USE_COMMA_DELIMITER: LazyLock<bool> = LazyLock::new(|| {
    let enable = parse_bool_from_env("USE_COMMA_DELIMITER", true);
    if enable && *TOKEN_DELIMITER == COMMA {
        false
    } else {
        enable
    }
});

pub static USE_PRI_REVERSE_PROXY: LazyLock<bool> =
    LazyLock::new(|| !PRI_REVERSE_PROXY_HOST.is_empty());

pub static USE_PUB_REVERSE_PROXY: LazyLock<bool> =
    LazyLock::new(|| !PUB_REVERSE_PROXY_HOST.is_empty());

macro_rules! def_cursor_api_url {
    ($name:ident, $api_host:expr, $path:expr) => {
        pub fn $name(is_pri: bool) -> &'static str {
            static URL_PRI: OnceLock<String> = OnceLock::new();
            static URL_PUB: OnceLock<String> = OnceLock::new();

            if is_pri {
                URL_PRI.get_or_init(|| {
                    let host = if *USE_PRI_REVERSE_PROXY {
                        PRI_REVERSE_PROXY_HOST.as_str()
                    } else {
                        $api_host
                    };
                    format!("https://{}{}", host, $path)
                })
            } else {
                URL_PUB.get_or_init(|| {
                    let host = if *USE_PUB_REVERSE_PROXY {
                        PUB_REVERSE_PROXY_HOST.as_str()
                    } else {
                        $api_host
                    };
                    format!("https://{}{}", host, $path)
                })
            }
        }
    };
}

def_cursor_api_url!(
    cursor_api2_chat_url,
    CURSOR_API2_HOST,
    "/aiserver.v1.AiService/StreamChat"
);

def_cursor_api_url!(
    cursor_api2_chat_web_url,
    CURSOR_API2_HOST,
    "/aiserver.v1.AiService/StreamChatWeb"
);

def_cursor_api_url!(
    cursor_api2_chat_models_url,
    CURSOR_API2_HOST,
    "/aiserver.v1.AiService/AvailableModels"
);

def_cursor_api_url!(
    cursor_api2_token_usage_url,
    CURSOR_API2_HOST,
    "/aiserver.v1.DashboardService/GetTokenUsage"
);

def_cursor_api_url!(
    cursor_api2_stripe_url,
    CURSOR_API2_HOST,
    "/auth/full_stripe_profile"
);

def_cursor_api_url!(cursor_usage_api_url, CURSOR_HOST, "/api/usage");

def_cursor_api_url!(cursor_user_api_url, CURSOR_HOST, "/api/auth/me");

def_cursor_api_url!(
    cursor_token_upgrade_url,
    CURSOR_HOST,
    "/api/auth/loginDeepCallbackControl"
);

def_cursor_api_url!(cursor_token_poll_url, CURSOR_API2_HOST, "/auth/poll");

static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let data_dir = parse_string_from_env("DATA_DIR", "data");
    let path = std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(data_dir);
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("无法创建数据目录");
    }
    path
});

pub(super) static CONFIG_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_DIR.join("config.bin"));

pub(super) static LOGS_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("logs.bin"));

pub(super) static TOKENS_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_DIR.join("tokens.bin"));

pub(super) static PROXIES_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_DIR.join("proxies.bin"));

pub static DEBUG: LazyLock<bool> = LazyLock::new(|| parse_bool_from_env("DEBUG", false));

// 使用环境变量 "DEBUG_LOG_FILE" 来指定日志文件路径，默认值为 "debug.log"
static DEBUG_LOG_FILE: LazyLock<String> =
    LazyLock::new(|| parse_string_from_env("DEBUG_LOG_FILE", "debug.log"));

// 使用 OnceCell 结合 Mutex 来异步初始化 LOG_FILE
static LOG_FILE: OnceCell<Mutex<tokio::fs::File>> = OnceCell::const_new();

pub(crate) async fn get_log_file() -> &'static Mutex<tokio::fs::File> {
    LOG_FILE
        .get_or_init(|| async {
            Mutex::new(
                tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&*DEBUG_LOG_FILE)
                    .await
                    .expect("无法打开日志文件"),
            )
        })
        .await
}

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if *$crate::app::lazy::DEBUG {
            let time = $crate::app::lazy::now_in_general_timezone().format("%Y-%m-%d %H:%M:%S").to_string();
            let log_message = format!("{} - {}", time, format!($($arg)*));
            use tokio::io::AsyncWriteExt as _;

            // 使用 tokio 的 spawn 在后台异步写入日志
            tokio::spawn(async move {
                let log_file = $crate::app::lazy::get_log_file().await;
                // 使用 MutexGuard 获取可变引用
                let mut file = log_file.lock().await;
                if let Err(err) = file.write_all(log_message.as_bytes()).await {
                    eprintln!("写入日志文件失败: {}", err);
                }
                if let Err(err) = file.write_all(b"\n").await {
                    eprintln!("写入换行符失败: {}", err);
                }
                // 可以选择在写入失败时 panic，或者忽略
                // panic!("写入日志文件失败: {}", err);
            });
        }
    };
}

// 请求日志相关常量
const DEFAULT_REQUEST_LOGS_LIMIT: usize = 100;
const MAX_REQUEST_LOGS_LIMIT: usize = 100000;

pub static REQUEST_LOGS_LIMIT: LazyLock<usize> = LazyLock::new(|| {
    std::cmp::min(
        parse_usize_from_env("REQUEST_LOGS_LIMIT", DEFAULT_REQUEST_LOGS_LIMIT),
        MAX_REQUEST_LOGS_LIMIT,
    )
});

pub static IS_NO_REQUEST_LOGS: LazyLock<bool> = LazyLock::new(|| *REQUEST_LOGS_LIMIT == 0);
pub static IS_UNLIMITED_REQUEST_LOGS: LazyLock<bool> =
    LazyLock::new(|| *REQUEST_LOGS_LIMIT == MAX_REQUEST_LOGS_LIMIT);

// TCP 和超时相关常量
const DEFAULT_TCP_KEEPALIVE: usize = 90;
const MAX_TCP_KEEPALIVE: u64 = 600;

pub static TCP_KEEPALIVE: LazyLock<u64> = LazyLock::new(|| {
    let keepalive = parse_usize_from_env("TCP_KEEPALIVE", DEFAULT_TCP_KEEPALIVE);
    u64::try_from(keepalive)
        .map(|t| t.min(MAX_TCP_KEEPALIVE))
        .unwrap_or(DEFAULT_TCP_KEEPALIVE as u64)
});

const DEFAULT_SERVICE_TIMEOUT: usize = 30;
const MAX_SERVICE_TIMEOUT: u64 = 600;

pub static SERVICE_TIMEOUT: LazyLock<u64> = LazyLock::new(|| {
    let timeout = parse_usize_from_env("SERVICE_TIMEOUT", DEFAULT_SERVICE_TIMEOUT);
    u64::try_from(timeout)
        .map(|t| t.min(MAX_SERVICE_TIMEOUT))
        .unwrap_or(DEFAULT_SERVICE_TIMEOUT as u64)
});

pub static REAL_USAGE: LazyLock<bool> = LazyLock::new(|| parse_bool_from_env("REAL_USAGE", false));

pub static SAFE_HASH: LazyLock<bool> = LazyLock::new(|| parse_bool_from_env("SAFE_HASH", true));
