use crate::{
    app::model::proxy_pool::ProxyPool,
    common::{
        model::{ApiStatus, userinfo::GetUserInfo},
        utils::{extract_token, get_new_token, get_token_profile},
    },
    core::constant::ERR_NODATA,
};
use axum::Json;

use super::token::TokenRequest;

pub async fn handle_user_info(Json(request): Json<TokenRequest>) -> Json<GetUserInfo> {
    let auth_token = match request.token {
        Some(token) => token,
        None => {
            return Json(GetUserInfo::Error {
                error: ERR_NODATA.to_string(),
            });
        }
    };

    let token = match extract_token(&auth_token) {
        Some(token) => token,
        None => {
            return Json(GetUserInfo::Error {
                error: ERR_NODATA.to_string(),
            });
        }
    };

    match get_token_profile(ProxyPool::get_general_client(), &token, false).await {
        Some(usage) => Json(GetUserInfo::Usage(Box::new(usage))),
        None => Json(GetUserInfo::Error {
            error: ERR_NODATA.to_string(),
        }),
    }
}

#[derive(serde::Serialize)]
pub struct TokenUpgradeResponse {
    status: ApiStatus,
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
}

pub async fn handle_token_upgrade(Json(request): Json<TokenRequest>) -> Json<TokenUpgradeResponse> {
    // 从请求头中获取并验证 auth token
    let auth_token = match request.token {
        Some(token) => token,
        None => {
            return Json(TokenUpgradeResponse {
                status: ApiStatus::Error,
                message: "未提供授权令牌",
                result: None,
            });
        }
    };

    let token = match extract_token(&auth_token) {
        Some(token) => token,
        None => {
            return Json(TokenUpgradeResponse {
                status: ApiStatus::Error,
                message: "无法解析授权令牌",
                result: None,
            });
        }
    };

    match get_new_token(ProxyPool::get_general_client(), &token, false).await {
        Some(token) => Json(TokenUpgradeResponse {
            status: ApiStatus::Success,
            message: "升级成功",
            result: Some(token),
        }),
        None => Json(TokenUpgradeResponse {
            status: ApiStatus::Failure,
            message: "升级失败",
            result: None,
        }),
    }
}
