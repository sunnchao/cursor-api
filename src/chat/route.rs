mod logs;
pub use logs::{handle_logs, handle_logs_post};
mod health;
pub use health::{handle_health, handle_root};
mod tokens;
pub use tokens::{
    handle_add_tokens, handle_basic_calibration, handle_delete_tokens, handle_get_checksum,
    handle_get_hash, handle_get_timestamp_header, handle_get_tokens, handle_reload_tokens,
    handle_tokens_page, handle_update_tokens,
};
mod profile;
pub use profile::handle_user_info;
mod config;
pub use config::{
    handle_about, handle_build_key, handle_build_key_page, handle_config_page, handle_env_example,
    handle_readme, handle_static,
};
mod api;
pub use api::handle_api_page;
