mod proxy_url;

use crate::app::lazy::{PROXIES_FILE_PATH, SERVICE_TIMEOUT, TCP_KEEPALIVE};
use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use proxy_url::StringUrl;
use reqwest::Client;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    str::FromStr,
    sync::LazyLock,
    time::Duration,
};

// 新的代理值常量
pub const NON_PROXY: &str = "non";
pub const SYS_PROXY: &str = "sys";

// 直接初始化PROXY_POOL为一个带有系统代理的基本实例
pub static PROXY_POOL: LazyLock<RwLock<ProxyPool>> = LazyLock::new(|| {
    let system_client = Client::builder()
        .https_only(true)
        .tcp_keepalive(Duration::from_secs(*TCP_KEEPALIVE))
        .connect_timeout(Duration::from_secs(*SERVICE_TIMEOUT))
        .build()
        .expect("创建默认系统客户端失败");

    RwLock::new(ProxyPool {
        proxies: HashMap::from([(SYS_PROXY.to_string(), SingleProxy::Sys)]),
        clients: HashMap::from([(SingleProxy::Sys, system_client.clone())]),
        general: Some(system_client),
    })
});

#[derive(Clone, Deserialize, Serialize, Archive, RkyvDeserialize, RkyvSerialize)]
pub struct Proxies {
    // name to proxy
    proxies: HashMap<String, SingleProxy>,
    general: String,
}

impl Default for Proxies {
    fn default() -> Self {
        Self::new()
    }
}

impl Proxies {
    pub fn new() -> Self {
        Self {
            proxies: HashMap::from([(SYS_PROXY.to_string(), SingleProxy::Sys)]),
            general: SYS_PROXY.to_string(),
        }
    }

    pub fn get_proxies(&self) -> &HashMap<String, SingleProxy> {
        &self.proxies
    }

    pub fn add_proxy(&mut self, name: String, proxy: SingleProxy) {
        self.proxies.insert(name, proxy);
    }

    pub fn remove_proxy(&mut self, name: &str) {
        self.proxies.remove(name);
    }

    pub fn set_general(&mut self, name: &str) {
        if self.proxies.contains_key(name) {
            self.general = name.to_string();
        }
    }

    pub fn get_general(&self) -> &str {
        &self.general
    }

    // 更新全局代理池
    pub fn update_global_pool(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取全局代理池的写锁
        let mut pool = PROXY_POOL.write();

        // 确保self.proxies至少有系统代理，且general有效
        if self.proxies.is_empty() {
            self.proxies.insert(SYS_PROXY.to_string(), SingleProxy::Sys);
            self.general = SYS_PROXY.to_string();
        } else if !self.proxies.contains_key(&self.general) {
            // general指向的代理不存在，更新为某个存在的代理
            self.general = self.proxies.keys().next().unwrap().clone();
        }

        // 1. 收集当前配置中的所有唯一代理
        let current_proxies: HashSet<&SingleProxy> = self.proxies.values().collect();

        // 2. 直接更新代理映射，避免克隆
        pool.proxies = self.proxies.clone();

        // 3. 更新客户端实例
        // 为新的代理配置创建客户端
        for proxy in &current_proxies {
            if !pool.clients.contains_key(proxy) {
                // 创建新的客户端
                match proxy {
                    SingleProxy::Non => {
                        pool.clients.insert(
                            SingleProxy::Non,
                            Client::builder()
                                .https_only(true)
                                .tcp_keepalive(Duration::from_secs(*TCP_KEEPALIVE))
                                .connect_timeout(Duration::from_secs(*SERVICE_TIMEOUT))
                                .no_proxy()
                                .build()
                                .expect("创建无代理客户端失败"),
                        );
                    }
                    SingleProxy::Sys => {
                        pool.clients.insert(
                            SingleProxy::Sys,
                            Client::builder()
                                .https_only(true)
                                .tcp_keepalive(Duration::from_secs(*TCP_KEEPALIVE))
                                .connect_timeout(Duration::from_secs(*SERVICE_TIMEOUT))
                                .build()
                                .expect("创建默认客户端失败"),
                        );
                    }
                    SingleProxy::Url(url) => {
                        pool.clients.insert(
                            (*proxy).clone(),
                            Client::builder()
                                .https_only(true)
                                .tcp_keepalive(Duration::from_secs(*TCP_KEEPALIVE))
                                .connect_timeout(Duration::from_secs(*SERVICE_TIMEOUT))
                                .proxy(url.as_proxy().expect("创建代理对象失败"))
                                .build()
                                .expect("创建代理客户端失败"),
                        );
                    }
                }
            }
        }

        // 4. 移除不再使用的客户端
        let to_remove: Vec<SingleProxy> = pool
            .clients
            .keys()
            .filter(|proxy| !current_proxies.contains(proxy))
            .cloned()
            .collect();

        for proxy in to_remove {
            pool.clients.remove(&proxy);
        }

        // 5. 设置通用客户端
        pool.general = Some(
            pool.clients
                .get(
                    self.proxies
                        .get(&self.general)
                        .expect("General proxy not found in proxy list"),
                )
                .expect("Client for general proxy not found in client pool")
                .clone(),
        );

        Ok(())
    }

    pub async fn save_proxies(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = rkyv::to_bytes::<_, 256>(self)?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&*PROXIES_FILE_PATH)?;

        if bytes.len() > usize::MAX / 2 {
            return Err("代理数据过大".into());
        }

        file.set_len(bytes.len() as u64)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap.copy_from_slice(&bytes);
        mmap.flush()?;

        Ok(())
    }

    pub async fn load_proxies() -> Result<Self, Box<dyn std::error::Error>> {
        let file = match OpenOptions::new().read(true).open(&*PROXIES_FILE_PATH) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::new());
            }
            Err(e) => return Err(Box::new(e)),
        };

        if file.metadata()?.len() > usize::MAX as u64 {
            return Err("代理文件过大".into());
        }

        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let archived = unsafe { rkyv::archived_root::<Self>(&mmap) };
        Ok(archived.deserialize(&mut rkyv::Infallible)?)
    }

    // 更新全局代理池并保存配置
    pub async fn update_and_save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 更新全局代理池
        self.update_global_pool()?;

        // 保存配置到文件
        self.save_proxies().await
    }
}

#[derive(Clone, Archive, RkyvDeserialize, RkyvSerialize, PartialEq, Eq, Hash)]
#[archive(compare(PartialEq))]
pub enum SingleProxy {
    Non,
    Sys,
    Url(StringUrl),
}

impl Serialize for SingleProxy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Non => serializer.serialize_str(NON_PROXY),
            Self::Sys => serializer.serialize_str(SYS_PROXY),
            Self::Url(url) => serializer.serialize_str(&url.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for SingleProxy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SingleProxyVisitor;

        impl serde::de::Visitor<'_> for SingleProxyVisitor {
            type Value = SingleProxy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing 'non', 'sys', or a valid URL")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    NON_PROXY => Ok(Self::Value::Non),
                    SYS_PROXY => Ok(Self::Value::Sys),
                    url_str => Ok(Self::Value::Url(
                        StringUrl::from_str(url_str)
                            .map_err(|e| E::custom(format!("Invalid URL: {e}")))?,
                    )),
                }
            }
        }

        deserializer.deserialize_str(SingleProxyVisitor)
    }
}

impl std::fmt::Display for SingleProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Non => write!(f, "{NON_PROXY}"),
            Self::Sys => write!(f, "{SYS_PROXY}"),
            Self::Url(url) => write!(f, "{url}"),
        }
    }
}

impl FromStr for SingleProxy {
    type Err = reqwest::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            NON_PROXY => Ok(Self::Non),
            SYS_PROXY => Ok(Self::Sys),
            url_str => Ok(Self::Url(StringUrl::from_str(url_str)?)),
        }
    }
}

pub struct ProxyPool {
    // 名称到代理配置的映射 - 类似于 Proxies 中的 proxies 字段
    proxies: HashMap<String, SingleProxy>,
    // 代理配置到客户端实例的映射 - 避免重复创建相同配置的客户端
    clients: HashMap<SingleProxy, Client>,
    // 通用客户端 - 用于未指定特定代理的请求
    general: Option<Client>,
}

/// ProxyPool 是系统内部使用的代理池实现，
/// 而 Proxies 是面向用户的配置结构。
///
/// ProxyPool 存在的目的：
/// 1. 优化相同代理配置的客户端管理，避免重复创建
/// 2. 提供高效的客户端查找机制
/// 3. 维护代理连接的生命周期
impl ProxyPool {
    // 获取客户端
    pub fn get_client(url: &str) -> Client {
        let pool = PROXY_POOL.read();

        // 先通过名称查找代理配置
        if let Some(proxy) = pool.proxies.get(url.trim()) {
            // 然后通过代理配置查找客户端
            if let Some(client) = pool.clients.get(proxy) {
                return client.clone();
            }
        }

        // 返回通用客户端或默认客户端
        pool.general
            .clone()
            .expect("general client should be initialized")
    }

    // 获取通用客户端
    pub fn get_general_client() -> Client {
        let pool = PROXY_POOL.read();
        pool.general
            .clone()
            .expect("general client should be initialized")
    }

    // 获取客户端或通用客户端
    #[inline]
    pub fn get_client_or_general(url: Option<&str>) -> Client {
        match url {
            Some(url) => Self::get_client(url),
            None => Self::get_general_client(),
        }
    }
}
