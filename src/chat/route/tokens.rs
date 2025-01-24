use crate::{
    app::{
        constant::{
            AUTHORIZATION_BEARER_PREFIX, CONTENT_TYPE_TEXT_HTML_WITH_UTF8,
            CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, ROUTE_TOKENS_PATH,
        },
        lazy::{AUTH_TOKEN, TOKEN_LIST_FILE},
        model::{
            AppConfig, AppState, PageContent, TokenAddRequestTokenInfo, TokenInfo,
            TokenUpdateRequest, TokensDeleteRequest, TokensDeleteResponse,
        },
    },
    common::{
        model::{error::ChatError, ApiStatus, ErrorResponse},
        utils::{
            extract_time, extract_time_ks, extract_user_id, generate_checksum_with_default,
            generate_checksum_with_repair, generate_hash, generate_timestamp_header, load_tokens,
            parse_token, validate_token, validate_token_and_checksum, write_tokens,
        },
    },
};
use axum::{
    extract::{Query, State},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderMap,
    },
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_get_hash() -> Response {
    let hash = generate_hash();

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8.parse().unwrap(),
    );

    (headers, hash).into_response()
}

#[derive(Deserialize)]
pub struct ChecksumQuery {
    #[serde(default)]
    pub checksum: Option<String>,
}

pub async fn handle_get_checksum(Query(query): Query<ChecksumQuery>) -> Response {
    let checksum = match query.checksum {
        None => generate_checksum_with_default(),
        Some(checksum) => generate_checksum_with_repair(&checksum),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8.parse().unwrap(),
    );

    (headers, checksum).into_response()
}

pub async fn handle_get_timestamp_header() -> Response {
    let timestamp_header = generate_timestamp_header();

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8.parse().unwrap(),
    );

    (headers, timestamp_header).into_response()
}

pub async fn handle_get_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let tokens = state.lock().await.token_infos.clone();
    let tokens_count = tokens.len();

    Ok(Json(TokenInfoResponse {
        status: ApiStatus::Success,
        tokens: Some(tokens),
        tokens_count,
        message: None,
    }))
}

#[derive(Serialize)]
pub struct TokenInfoResponse {
    pub status: ApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<TokenInfo>>,
    pub tokens_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn handle_reload_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 重新加载 tokens
    let tokens = load_tokens();
    let tokens_count = tokens.len();

    // 更新应用状态
    {
        let mut state = state.lock().await;
        state.token_infos = tokens;
    }

    Ok(Json(TokenInfoResponse {
        status: ApiStatus::Success,
        tokens: None,
        tokens_count,
        message: Some("Token list has been reloaded".to_string()),
    }))
}

pub async fn handle_update_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<TokenUpdateRequest>,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token_list_file = TOKEN_LIST_FILE.as_str();

    std::fs::write(&token_list_file, &request.tokens)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 重新加载 tokens
    let token_infos = load_tokens();
    let tokens_count = token_infos.len();

    // 更新应用状态
    {
        let mut state = state.lock().await;
        state.token_infos = token_infos;
    }

    Ok(Json(TokenInfoResponse {
        status: ApiStatus::Success,
        tokens: None,
        tokens_count,
        message: Some("Token files have been updated and reloaded".to_string()),
    }))
}

pub async fn handle_add_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<Vec<TokenAddRequestTokenInfo>>,
) -> Result<Json<TokenInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ))?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ));
    }

    let token_list_file = TOKEN_LIST_FILE.as_str();

    // 获取当前的 tokens 并创建新的 token_infos
    let mut token_infos = {
        let state = state.lock().await;
        state.token_infos.clone()
    };

    // 创建现有token的集合
    let existing_tokens: std::collections::HashSet<_> =
        token_infos.iter().map(|info| info.token.as_str()).collect();

    // 预分配容量
    let mut new_tokens = Vec::with_capacity(request.len());

    // 处理新的tokens
    for token_info in request {
        let parsed_token = parse_token(&token_info.token);
        if !existing_tokens.contains(parsed_token.as_str()) && validate_token(&parsed_token) {
            new_tokens.push(TokenInfo {
                token: parsed_token,
                // 如果提供了checksum就使用提供的，否则生成新的
                checksum: token_info
                    .checksum
                    .as_deref()
                    .map(generate_checksum_with_repair)
                    .unwrap_or_else(generate_checksum_with_default),
                profile: None,
            });
        }
    }

    // 如果有新tokens才进行后续操作
    if !new_tokens.is_empty() {
        // 预分配足够的容量
        token_infos.reserve(new_tokens.len());
        token_infos.extend(new_tokens);

        // 写入文件
        write_tokens(&token_infos, token_list_file).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some("Failed to update token list file".to_string()),
                    message: Some("无法更新token list文件".to_string()),
                }),
            )
        })?;

        // 获取最终的tokens数量（在更新状态之前）
        let tokens_count = token_infos.len();

        // 更新应用状态
        {
            let mut state = state.lock().await;
            state.token_infos = token_infos;
        }

        Ok(Json(TokenInfoResponse {
            status: ApiStatus::Success,
            tokens: None,
            tokens_count,
            message: Some("New tokens have been added and reloaded".to_string()),
        }))
    } else {
        // 如果没有新tokens，使用原始数量
        let tokens_count = token_infos.len();

        Ok(Json(TokenInfoResponse {
            status: ApiStatus::Success,
            tokens: None,
            tokens_count,
            message: Some("No new tokens were added".to_string()),
        }))
    }
}

pub async fn handle_delete_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<TokensDeleteRequest>,
) -> Result<Json<TokensDeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ))?;

    if auth_header != AUTH_TOKEN.as_str() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ));
    }

    let token_infos = state.lock().await.token_infos.clone();
    let original_count = token_infos.len(); // 提前存储原始长度

    // 获取token_list文件路径
    let token_list_file = TOKEN_LIST_FILE.as_str();

    // 创建要删除的tokens的HashSet，提高查找效率
    let tokens_to_delete: std::collections::HashSet<_> = request.tokens.iter().collect();

    // 如果需要的话计算 failed_tokens
    let failed_tokens = if request.expectation.needs_failed_tokens() {
        Some(
            request
                .tokens
                .iter()
                .filter(|token| !token_infos.iter().any(|info| &info.token == *token))
                .cloned()
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    // 预分配容量并过滤掉要删除的tokens
    let estimated_capacity = original_count.saturating_sub(tokens_to_delete.len());
    let mut filtered_token_infos = Vec::with_capacity(estimated_capacity);

    // 一次性过滤tokens
    for info in token_infos {
        if !tokens_to_delete.contains(&info.token) {
            filtered_token_infos.push(info);
        }
    }

    // 如果有tokens被删除才进行更新操作
    if filtered_token_infos.len() < original_count {
        // 写入文件
        write_tokens(&filtered_token_infos, token_list_file).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some("Failed to update token list file".to_string()),
                    message: Some("无法更新token list文件".to_string()),
                }),
            )
        })?;

        // 如果需要的话计算 updated_tokens
        let updated_tokens = if request.expectation.needs_updated_tokens() {
            Some(
                filtered_token_infos
                    .iter()
                    .map(|info| info.token.clone())
                    .collect(),
            )
        } else {
            None
        };

        // 更新状态
        {
            let mut state = state.lock().await;
            state.token_infos = filtered_token_infos;
        }

        Ok(Json(TokensDeleteResponse {
            status: ApiStatus::Success,
            updated_tokens,
            failed_tokens,
        }))
    } else {
        // 如果没有tokens被删除
        Ok(Json(TokensDeleteResponse {
            status: ApiStatus::Success,
            updated_tokens: if request.expectation.needs_updated_tokens() {
                Some(
                    filtered_token_infos
                        .iter()
                        .map(|info| info.token.clone())
                        .collect(),
                )
            } else {
                None
            },
            failed_tokens,
        }))
    }
}

pub async fn handle_tokens_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_TOKENS_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/tokens.min.html").to_string())
            .unwrap(),
        PageContent::Text(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8)
            .body(content.clone())
            .unwrap(),
        PageContent::Html(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(content.clone())
            .unwrap(),
    }
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct BasicCalibrationResponse {
    pub status: ApiStatus,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_at: Option<String>,
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
                message: Some("未提供授权令牌".to_string()),
                user_id: None,
                create_at: None,
                checksum_time: None,
            })
        }
    };

    // 校验 token 和 checksum
    let (token, checksum) = match validate_token_and_checksum(&auth_token) {
        Some(parts) => parts,
        None => {
            return Json(BasicCalibrationResponse {
                status: ApiStatus::Error,
                message: Some("无效令牌或无效校验和".to_string()),
                user_id: None,
                create_at: None,
                checksum_time: None,
            })
        }
    };

    // 提取用户ID和创建时间
    let user_id = extract_user_id(&token);
    let create_at = extract_time(&token).map(|dt| dt.to_string());
    let checksum_time = extract_time_ks(&checksum[..8]);

    // 返回校验结果
    Json(BasicCalibrationResponse {
        status: ApiStatus::Success,
        message: Some("校验成功".to_string()),
        user_id,
        create_at,
        checksum_time,
    })
}
