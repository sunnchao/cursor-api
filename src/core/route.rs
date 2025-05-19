mod logs;
pub use logs::{handle_logs, handle_logs_post};
mod health;
pub use health::{handle_health, handle_root};
mod token;
pub use token::{handle_basic_calibration, handle_build_key};
mod tokens;
pub use tokens::{
    handle_add_tokens, handle_delete_tokens, handle_get_token_tags, handle_get_tokens,
    handle_get_tokens_by_tag, handle_set_token_tags, handle_set_tokens, handle_set_tokens_status,
    handle_update_tokens_profile, handle_upgrade_tokens,
};
mod checksum;
pub use checksum::{handle_get_checksum, handle_get_hash, handle_get_timestamp_header};
mod profile;
pub use profile::{handle_token_upgrade, handle_user_info};
mod proxies;
pub use proxies::{
    handle_add_proxy, handle_delete_proxies, handle_get_proxies, handle_set_general_proxy,
    handle_set_proxies,
};
mod page;
pub use page::{
    handle_about, handle_api_page, handle_build_key_page, handle_config_page, handle_env_example,
    handle_proxies_page, handle_readme, handle_static, handle_tokens_page,
};
