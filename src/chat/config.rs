use crate::AppConfig;

include!(concat!(env!("OUT_DIR"), "/key.rs"));

impl KeyConfig {
    pub fn new_with_global() -> Self {
        Self {
            auth_token: None,
            enable_stream_check: Some(AppConfig::get_stream_check()),
            include_stop_stream: Some(AppConfig::get_stop_stream()),
            disable_vision: Some(AppConfig::get_vision_ability().is_none()),
            enable_slow_pool: Some(AppConfig::get_slow_pool()),
            usage_check_models: None,
        }
    }

    pub fn copy_without_auth_token(&self, config: &mut Self) {
        if self.enable_stream_check.is_some() {
            config.enable_stream_check = self.enable_stream_check;
        }
        if self.include_stop_stream.is_some() {
            config.include_stop_stream = self.include_stop_stream;
        }
        if self.disable_vision.is_some() {
            config.disable_vision = self.disable_vision;
        }
        if self.enable_slow_pool.is_some() {
            config.enable_slow_pool = self.enable_slow_pool;
        }
        if self.usage_check_models.is_some() {
            config.usage_check_models = self.usage_check_models.clone();
        }
    }
}
