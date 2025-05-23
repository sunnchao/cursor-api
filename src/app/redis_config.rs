use serde::{Deserialize, Serialize};
use std::env;
use std::sync::RwLock;
use lazy_static::lazy_static;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
    pub pool_size: u32,
    pub timeout: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "redis-8e090d6-chaowang-c39a.h.aivencloud.com".to_string(),
            port: 20346,
            password: Some("AVNS_dvyneqowB90t4acNcz0".to_string()),
            db: 0,
            pool_size: 10,
            timeout: 5,
        }
    }
}

impl RedisConfig {
    pub fn from_env() -> RedisConfig {
        let host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()).parse().unwrap_or(6379);
        let password = env::var("REDIS_PASSWORD").ok();
        let db = env::var("REDIS_DB").unwrap_or_else(|_| "0".to_string()).parse().unwrap_or(0);
        let pool_size = env::var("REDIS_POOL_SIZE").unwrap_or_else(|_| "10".to_string()).parse().unwrap_or(10);
        let timeout = env::var("REDIS_TIMEOUT").unwrap_or_else(|_| "5".to_string()).parse().unwrap_or(5);

        RedisConfig {
            host,
            port,
            password,
            db,
            pool_size,
            timeout,
        }
    }

    pub fn new(host: String, port: u16, password: Option<String>, db: u8, pool_size: u32, timeout: u64) -> Self {
        Self {
            host,
            port,
            password,
            db,
            pool_size,
            timeout,
        }
    }

    pub fn get_connection_string(&self) -> String {
        match &self.password {
            Some(pass) => format!("redis://:{}@{}:{}/{}", pass, self.host, self.port, self.db),
            None => format!("redis://{}:{}/{}", self.host, self.port, self.db),
        }
    }

    pub fn update_config(&self) {
        let mut config = REDIS_CONFIG.write().unwrap();
        *config = self.clone();
    }

    pub fn get_config() -> RedisConfig {
        REDIS_CONFIG.read().unwrap().clone()
    }
}

lazy_static! {
    pub static ref REDIS_CONFIG: RwLock<RedisConfig> = RwLock::new(RedisConfig::from_env());
}
