use super::{
    ApiStatus, DeleteResponseExpectation,
    proxy_pool::{Proxies, SingleProxy},
};
use serde::{Deserialize, Serialize};

// 代理信息响应
#[derive(Serialize)]
pub struct ProxyInfoResponse {
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Proxies>,
    pub proxies_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general_proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// 更新代理配置请求
#[derive(Deserialize)]
pub struct ProxyUpdateRequest {
    pub proxies: Proxies,
}

// 添加代理请求
#[derive(Deserialize)]
pub struct ProxyAddRequest {
    pub proxies: std::collections::HashMap<String, SingleProxy>,
}

// 删除代理请求
#[derive(Deserialize)]
pub struct ProxiesDeleteRequest {
    #[serde(default)]
    pub names: std::collections::HashSet<String>,
    #[serde(default)]
    pub expectation: DeleteResponseExpectation,
}

// 删除代理响应
#[derive(Serialize)]
pub struct ProxiesDeleteResponse {
    pub status: ApiStatus,
    pub updated_proxies: Option<Proxies>,
    pub failed_names: Option<Vec<String>>,
}

// 设置通用代理请求
#[derive(Deserialize)]
pub struct SetGeneralProxyRequest {
    pub name: String,
}
