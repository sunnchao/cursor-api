pub mod config;
pub mod error;
pub mod health;
pub mod token;
pub mod tri;
pub mod userinfo;

use std::borrow::Cow;

use config::ConfigData;

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    Healthy,
    Success,
    Error,
    Failure,
}

// #[derive(Serialize)]
// #[serde(untagged)]
// pub enum ApiResponse {
//     HealthCheck(HealthCheckResponse),
//     ConfigData(NormalResponse<ConfigData>),
//     Error(ErrorResponse),
// }

// impl ApiResponse {
//     pub fn to_string(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }
// }

#[derive(Serialize)]
pub struct NormalResponse<T> {
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Cow<'static, str>>,
}

impl std::fmt::Display for NormalResponse<ConfigData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

// #[derive(Serialize)]
// pub struct NormalResponseNoData {
//     pub status: ApiStatus,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub message: Option<String>,
// }

#[derive(Serialize)]
pub struct ErrorResponse {
    // status -> 成功 / 失败
    pub status: ApiStatus,
    // HTTP 请求的状态码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
    // HTTP 请求的错误码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Cow<'static, str>>,
    // 错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Cow<'static, str>>,
}
