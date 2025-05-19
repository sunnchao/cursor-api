use crate::{
    app::{
        constant::AUTHORIZATION_BEARER_PREFIX,
        lazy::{AUTH_TOKEN, KEY_PREFIX},
        model::{AppConfig, BuildKeyRequest, BuildKeyResponse, UsageCheckModelType},
    },
    common::{
        model::ApiStatus,
        utils::{
            JwtTime, extract_time, extract_time_ks, extract_user_id, to_base64, token_to_tokeninfo,
            validate_token_and_checksum,
        },
    },
    core::config::{KeyConfig, key_config},
};
use axum::{
    Json,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use prost::Message as _;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct BasicCalibrationResponse {
    pub status: ApiStatus,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<JwtTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_time: Option<u64>,
}

pub async fn handle_basic_calibration(
    Json(request): Json<TokenRequest>,
) -> Json<BasicCalibrationResponse> {
    // 从请求头中获取并验证 auth token
    let auth_token = match request.token {
        Some(token) => token,
        None => {
            return Json(BasicCalibrationResponse {
                status: ApiStatus::Error,
                message: "未提供授权令牌",
                user_id: None,
                time: None,
                checksum_time: None,
            });
        }
    };

    // 校验 token 和 checksum
    let (token, checksum) = match validate_token_and_checksum(&auth_token) {
        Some(parts) => parts,
        None => {
            return Json(BasicCalibrationResponse {
                status: ApiStatus::Error,
                message: "无效令牌或无效校验和",
                user_id: None,
                time: None,
                checksum_time: None,
            });
        }
    };

    // 提取用户ID和创建时间
    let user_id = extract_user_id(&token);
    let time = extract_time(&token);
    let checksum_time = extract_time_ks(&checksum[..8]);

    // 返回校验结果
    Json(BasicCalibrationResponse {
        status: ApiStatus::Success,
        message: "校验成功",
        user_id,
        time,
        checksum_time,
    })
}

pub async fn handle_build_key(
    headers: HeaderMap,
    Json(request): Json<BuildKeyRequest>,
) -> (StatusCode, Json<BuildKeyResponse>) {
    // 验证认证令牌
    if AppConfig::is_share() {
        let auth_header = headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX));

        if auth_header.is_none_or(|h| !AppConfig::share_token_eq(h) && h != AUTH_TOKEN.as_str()) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(BuildKeyResponse::Error("Unauthorized")),
            );
        }
    }

    // 验证并解析 auth_token
    let token_info = match token_to_tokeninfo(&request.auth_token, request.proxy_name) {
        Some(info) => info,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(BuildKeyResponse::Error("Invalid auth token")),
            );
        }
    };

    // 构建 proto 消息
    let mut key_config = KeyConfig {
        auth_token: Some(token_info),
        disable_vision: request.disable_vision,
        enable_slow_pool: request.enable_slow_pool,
        usage_check_models: None,
        include_web_references: request.include_web_references,
    };

    if let Some(usage_check_models) = request.usage_check_models {
        let usage_check = key_config::UsageCheckModel {
            r#type: match usage_check_models.model_type {
                UsageCheckModelType::Default => key_config::usage_check_model::Type::Default as i32,
                UsageCheckModelType::Disabled => {
                    key_config::usage_check_model::Type::Disabled as i32
                }
                UsageCheckModelType::All => key_config::usage_check_model::Type::All as i32,
                UsageCheckModelType::Custom => key_config::usage_check_model::Type::Custom as i32,
            },
            model_ids: if matches!(usage_check_models.model_type, UsageCheckModelType::Custom) {
                usage_check_models
                    .model_ids
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                Vec::new()
            },
        };
        key_config.usage_check_models = Some(usage_check);
    }

    // 序列化
    let encoded = key_config.encode_to_vec();

    let key = format!("{}{}", *KEY_PREFIX, to_base64(&encoded));

    (StatusCode::OK, Json(BuildKeyResponse::Key(key)))
}
