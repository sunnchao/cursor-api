use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use rkyv::{Deserialize as _, archived_root};
use std::{fs::OpenOptions, sync::LazyLock};

use crate::{
    app::{
        constant::{
            EMPTY_STRING, ERR_INVALID_PATH, ROUTE_ABOUT_PATH, ROUTE_API_PATH, ROUTE_BUILD_KEY_PATH,
            ROUTE_CONFIG_PATH, ROUTE_LOGS_PATH, ROUTE_PROXIES_PATH, ROUTE_README_PATH,
            ROUTE_ROOT_PATH, ROUTE_SHARED_JS_PATH, ROUTE_SHARED_STYLES_PATH, ROUTE_TOKENS_PATH,
        },
        lazy::CONFIG_FILE_PATH,
    },
    common::utils::{parse_bool_from_env, parse_string_from_env},
};

use super::{PageContent, Pages, UsageCheck, VisionAbility};

// 静态配置
#[derive(Default, Clone)]
pub struct AppConfig {
    vision_ability: VisionAbility,
    slow_pool: bool,
    long_context: bool,
    pages: Pages,
    usage_check: UsageCheck,
    dynamic_key: bool,
    share_token: String,
    is_share: bool,
    web_refs: bool,
}

// 全局配置实例
static APP_CONFIG: LazyLock<RwLock<AppConfig>> =
    LazyLock::new(|| RwLock::new(AppConfig::default()));

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
        config.vision_ability =
            VisionAbility::from_str(&parse_string_from_env("VISION_ABILITY", EMPTY_STRING));
        config.slow_pool = parse_bool_from_env("ENABLE_SLOW_POOL", false);
        config.long_context = parse_bool_from_env("ENABLE_LONG_CONTEXT", false);
        config.usage_check =
            UsageCheck::from_str(&parse_string_from_env("USAGE_CHECK", EMPTY_STRING));
        config.dynamic_key = parse_bool_from_env("DYNAMIC_KEY", false);
        config.share_token = parse_string_from_env("SHARED_TOKEN", EMPTY_STRING);
        config.is_share = !config.share_token.is_empty();
        config.web_refs = parse_bool_from_env("INCLUDE_WEB_REFERENCES", false)
    }

    config_methods! {
        slow_pool: bool, false;
        long_context: bool, false;
        dynamic_key: bool, false;
        web_refs: bool, false;
        vision_ability: VisionAbility, VisionAbility::default();
    }

    config_methods_clone! {
        usage_check: UsageCheck, UsageCheck::default();
    }

    pub fn get_share_token() -> String {
        APP_CONFIG.read().share_token.clone()
    }

    pub fn share_token_eq(s: &str) -> bool {
        APP_CONFIG.read().share_token == s
    }

    pub fn update_share_token(value: String) {
        if Self::share_token_eq(&value) {
            let mut config = APP_CONFIG.write();
            config.share_token = value;
            config.is_share = !config.share_token.is_empty();
        }
    }

    pub fn reset_share_token() {
        if !APP_CONFIG.read().share_token.is_empty() {
            let mut config = APP_CONFIG.write();
            config.share_token = String::new();
            config.is_share = false;
        }
    }

    pub fn get_page_content(path: &str) -> Option<PageContent> {
        match path {
            ROUTE_ROOT_PATH => Some(APP_CONFIG.read().pages.root_content.clone()),
            ROUTE_LOGS_PATH => Some(APP_CONFIG.read().pages.logs_content.clone()),
            ROUTE_CONFIG_PATH => Some(APP_CONFIG.read().pages.config_content.clone()),
            ROUTE_TOKENS_PATH => Some(APP_CONFIG.read().pages.tokens_content.clone()),
            ROUTE_PROXIES_PATH => Some(APP_CONFIG.read().pages.proxies_content.clone()),
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
            ROUTE_TOKENS_PATH => config.pages.tokens_content = content,
            ROUTE_PROXIES_PATH => config.pages.proxies_content = content,
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
            ROUTE_TOKENS_PATH => config.pages.tokens_content = PageContent::default(),
            ROUTE_PROXIES_PATH => config.pages.proxies_content = PageContent::default(),
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

    pub fn save_config() -> Result<(), Box<dyn std::error::Error>> {
        let pages = APP_CONFIG.read().pages.clone();
        let bytes = rkyv::to_bytes::<_, 256>(&pages)?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&*CONFIG_FILE_PATH)?;

        // 添加大小检查
        if bytes.len() > usize::MAX / 2 {
            return Err("配置数据过大".into());
        }

        file.set_len(bytes.len() as u64)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap.copy_from_slice(&bytes);
        mmap.flush()?;

        Ok(())
    }

    pub fn load_saved_config() -> Result<(), Box<dyn std::error::Error>> {
        let file = match OpenOptions::new().read(true).open(&*CONFIG_FILE_PATH) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(e) => return Err(Box::new(e)),
        };

        // 添加文件大小检查
        if file.metadata()?.len() > usize::MAX as u64 {
            return Err("配置文件过大".into());
        }

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        let archived = unsafe { archived_root::<Pages>(&mmap) };
        let pages = archived.deserialize(&mut rkyv::Infallible)?;
        let mut config = APP_CONFIG.write();
        config.pages = pages;

        Ok(())
    }
}
