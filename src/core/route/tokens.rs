use crate::{
    app::model::{
        AppState, CommonResponse, TokenAddRequest, TokenInfo, TokenInfoResponse, TokenManager,
        TokenStatusSetRequest, TokenTagsUpdateRequest, TokenUpdateRequest, TokensDeleteRequest,
        TokensDeleteResponse,
    },
    common::{
        model::{ApiStatus, ErrorResponse, NormalResponse},
        utils::{
            generate_checksum_with_default, generate_checksum_with_repair, generate_hash,
            parse_token, validate_token,
        },
    },
};
use axum::{Json, extract::State, http::StatusCode};
use std::{borrow::Cow, collections::HashSet, sync::Arc};
use tokio::sync::Mutex;

pub async fn handle_get_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    let state = state.lock().await;
    let tokens = state.token_manager.tokens.clone();
    let tokens_count = tokens.len();

    Ok(Json(TokenInfoResponse {
        status: ApiStatus::Success,
        tokens: Some(tokens),
        tokens_count,
        message: None,
    }))
}

pub async fn handle_set_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(tokens): Json<TokenUpdateRequest>,
) -> Result<Json<TokenInfoResponse>, StatusCode> {
    // 创建新的 TokenManager
    let token_manager = TokenManager::new(tokens);
    let tokens_count = token_manager.tokens.len();

    // 保存到文件
    token_manager
        .save_tokens()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 设置应用状态
    {
        let mut state = state.lock().await;
        state.token_manager = token_manager;
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
    Json(request): Json<TokenAddRequest>,
) -> Result<Json<TokenInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取当前的 token_manager
    let mut token_manager = {
        let state = state.lock().await;
        state.token_manager.clone()
    };

    // 创建现有token的集合
    let existing_tokens: std::collections::HashSet<_> = token_manager
        .tokens
        .iter()
        .map(|info| info.token.as_str())
        .collect();

    // 处理新的tokens
    let mut new_tokens = Vec::with_capacity(request.tokens.len());
    for token_info in request.tokens {
        let parsed_token = parse_token(&token_info.token);
        if !existing_tokens.contains(parsed_token.as_str()) && validate_token(&parsed_token) {
            new_tokens.push(TokenInfo {
                token: parsed_token,
                checksum: token_info
                    .checksum
                    .as_deref()
                    .map(generate_checksum_with_repair)
                    .unwrap_or_else(generate_checksum_with_default),
                status: request.status,
                client_key: Some(generate_hash()),
                profile: None,
                tags: request.tags.clone(),
            });
        }
    }

    // 如果有新tokens才进行后续操作
    if !new_tokens.is_empty() {
        // 添加新tokens
        token_manager.tokens.extend(new_tokens);
        let tokens_count = token_manager.tokens.len();

        // 设置全局标签
        if let Some(ref tags) = request.tags {
            token_manager.update_global_tags(tags);
        }

        // 保存到文件
        token_manager.save_tokens().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some(Cow::Borrowed("Failed to save token list")),
                    message: Some(Cow::Borrowed("无法保存token list")),
                }),
            )
        })?;

        // 设置应用状态
        {
            let mut state = state.lock().await;
            state.token_manager = token_manager;
        }

        Ok(Json(TokenInfoResponse {
            status: ApiStatus::Success,
            tokens: None,
            tokens_count,
            message: Some("New tokens have been added and reloaded".to_string()),
        }))
    } else {
        // 如果没有新tokens，返回当前状态
        let tokens_count = token_manager.tokens.len();

        Ok(Json(TokenInfoResponse {
            status: ApiStatus::Success,
            tokens: Some(token_manager.tokens),
            tokens_count,
            message: Some("No new tokens were added".to_string()),
        }))
    }
}

pub async fn handle_delete_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<TokensDeleteRequest>,
) -> Result<Json<TokensDeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取当前的 token_manager
    let mut token_manager = {
        let state = state.lock().await;
        state.token_manager.clone()
    };

    // 创建要删除的tokens的HashSet，提高查找效率
    let tokens_to_delete: std::collections::HashSet<_> = request.tokens.iter().collect();

    // 如果需要的话计算 failed_tokens
    let failed_tokens = if request.expectation.needs_failed_tokens() {
        Some(
            request
                .tokens
                .iter()
                .filter(|token| {
                    !token_manager
                        .tokens
                        .iter()
                        .any(|token_info| token_info.token == **token)
                })
                .cloned()
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    let original_count: usize = token_manager.tokens.len();

    // 从每个分组中删除指定的tokens
    token_manager
        .tokens
        .retain(|token_info| !tokens_to_delete.contains(&token_info.token));

    let new_count: usize = token_manager.tokens.len();

    // 如果有tokens被删除才进行设置操作
    if new_count < original_count {
        // 保存到文件
        token_manager.save_tokens().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some(Cow::Borrowed("Failed to save token list")),
                    message: Some(Cow::Borrowed("无法保存token list")),
                }),
            )
        })?;

        // 如果需要的话计算 updated_tokens
        let updated_tokens = if request.expectation.needs_updated_tokens() {
            Some(
                token_manager
                    .tokens
                    .iter()
                    .map(|t| t.token.clone())
                    .collect(),
            )
        } else {
            None
        };

        // 设置状态
        {
            let mut state = state.lock().await;
            state.token_manager = token_manager;
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
                    token_manager
                        .tokens
                        .iter()
                        .map(|t| t.token.clone())
                        .collect(),
                )
            } else {
                None
            },
            failed_tokens,
        }))
    }
}

pub async fn handle_set_token_tags(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<TokenTagsUpdateRequest>,
) -> Result<Json<CommonResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取并设置 token_manager
    {
        let mut state = state.lock().await;
        if let Err(e) = state
            .token_manager
            .update_tokens_tags(&request.tokens, request.tags)
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some(Cow::Owned(e.to_string())),
                    message: Some(Cow::Borrowed("设置标签失败")),
                }),
            ));
        }

        // 保存更改
        if (state.token_manager.save_tokens().await).is_err() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some(Cow::Borrowed("Failed to save token tags")),
                    message: Some(Cow::Borrowed("无法保存标签信息")),
                }),
            ));
        }
    }

    Ok(Json(CommonResponse {
        status: ApiStatus::Success,
        message: Some("标签设置成功".to_string()),
    }))
}

pub async fn handle_update_tokens_profile(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(tokens): Json<HashSet<String>>,
) -> Result<Json<CommonResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 验证请求
    if tokens.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("No tokens provided")),
                message: Some(Cow::Borrowed("未提供任何令牌")),
            }),
        ));
    }

    // 获取当前的 token_manager
    let mut state_guard = state.lock().await;
    let token_manager = &mut state_guard.token_manager;

    // 批量设置tokens的profile
    let mut updated_count: u32 = 0;
    let mut failed_count: u32 = 0;

    for token in &tokens {
        // 验证token是否在token_manager中存在
        if let Some(token_idx) = token_manager
            .tokens
            .iter()
            .position(|info| info.token == *token)
        {
            // 获取profile
            if let Some(profile) = crate::common::utils::get_token_profile(
                token_manager.tokens[token_idx].get_client(),
                token,
                true,
            )
            .await
            {
                // 设置profile
                token_manager.tokens[token_idx].profile = Some(profile);
                updated_count += 1;
            } else {
                failed_count += 1;
            }
        } else {
            failed_count += 1;
        }
    }

    // 保存更改
    if updated_count > 0 && token_manager.save_tokens().await.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("Failed to save token profiles")),
                message: Some(Cow::Borrowed("无法保存令牌配置数据")),
            }),
        ));
    }

    let message = format!("已更新{updated_count}个令牌配置, {failed_count}个令牌更新失败");

    Ok(Json(CommonResponse {
        status: ApiStatus::Success,
        message: Some(message),
    }))
}

pub async fn handle_upgrade_tokens(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(tokens): Json<HashSet<String>>,
) -> Result<Json<CommonResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 验证请求
    if tokens.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("No tokens provided")),
                message: Some(Cow::Borrowed("未提供任何令牌")),
            }),
        ));
    }

    // 获取当前的 token_manager
    let mut state_guard = state.lock().await;
    let token_manager = &mut state_guard.token_manager;

    // 批量设置tokens的profile
    let mut updated_count: u32 = 0;
    let mut failed_count: u32 = 0;

    for token in &tokens {
        if let Some(token_idx) = token_manager
            .tokens
            .iter()
            .position(|info| info.token == *token)
        {
            if let Some(new_token) = crate::common::utils::get_new_token(
                token_manager.tokens[token_idx].get_client(),
                token,
                true,
            )
            .await
            {
                token_manager.tokens[token_idx].token = new_token;
                updated_count += 1;
            } else {
                failed_count += 1;
            }
        } else {
            failed_count += 1;
        }
    }

    // 保存更改
    if updated_count > 0 && token_manager.save_tokens().await.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("Failed to save tokens")),
                message: Some(Cow::Borrowed("无法保存令牌数据")),
            }),
        ));
    }

    let message = format!("已升级{updated_count}个令牌, {failed_count}个令牌升级失败");

    Ok(Json(CommonResponse {
        status: ApiStatus::Success,
        message: Some(message),
    }))
}

pub async fn handle_set_tokens_status(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<TokenStatusSetRequest>,
) -> Result<Json<CommonResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 验证请求
    if request.tokens.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("No tokens provided")),
                message: Some(Cow::Borrowed("未提供任何令牌")),
            }),
        ));
    }

    // 获取当前的 token_manager
    let mut state_guard = state.lock().await;
    let token_manager = &mut state_guard.token_manager;

    // 批量设置tokens的profile
    let mut updated_count: u32 = 0;
    let mut failed_count: u32 = 0;

    for token in &request.tokens {
        // 验证token是否在token_manager中存在
        if let Some(token_idx) = token_manager
            .tokens
            .iter()
            .position(|info| info.token == *token)
        {
            token_manager.tokens[token_idx].status = request.status;
            updated_count += 1;
        } else {
            failed_count += 1;
        }
    }

    // 保存更改
    if updated_count > 0 && token_manager.save_tokens().await.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("Failed to save token statuses")),
                message: Some(Cow::Borrowed("无法保存令牌状态数据")),
            }),
        ));
    }

    let message = format!("已设置{updated_count}个令牌状态, {failed_count}个令牌设置失败");

    Ok(Json(CommonResponse {
        status: ApiStatus::Success,
        message: Some(message),
    }))
}

pub async fn handle_get_token_tags(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<NormalResponse<Vec<String>>>, StatusCode> {
    let state = state.lock().await;
    let tags: Vec<_> = state.token_manager.tags.iter().cloned().collect();
    let len = tags.len();

    Ok(Json(NormalResponse {
        status: ApiStatus::Success,
        data: Some(tags),
        message: Some(Cow::Owned(format!("获取到{len}个标签"))),
    }))
}

pub async fn handle_get_tokens_by_tag(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(tag): Json<String>,
) -> Result<Json<TokenInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    let state = state.lock().await;

    match state.token_manager.get_tokens_by_tag(&tag) {
        Ok(tokens) => {
            let tokens_vec = tokens
                .iter()
                .map(|&t| t.clone())
                .collect::<Vec<TokenInfo>>();
            let tokens_count = tokens_vec.len();

            Ok(Json(TokenInfoResponse {
                status: ApiStatus::Success,
                tokens: Some(tokens_vec),
                tokens_count,
                message: Some(format!("获取到{tokens_count}个标签为{tag}的令牌")),
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Owned(e.to_string())),
                message: Some(Cow::Owned(format!("标签\"{tag}\"不存在"))),
            }),
        )),
    }
}
