use crate::{
    app::model::{
        AppState, CommonResponse, ProxiesDeleteRequest, ProxiesDeleteResponse, ProxyAddRequest,
        ProxyInfoResponse, ProxyUpdateRequest, SetGeneralProxyRequest,
    },
    common::model::{ApiStatus, ErrorResponse},
};
use axum::{Json, extract::State, http::StatusCode};
use std::{borrow::Cow, sync::Arc};
use tokio::sync::Mutex;

// 获取所有代理配置
pub async fn handle_get_proxies(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<ProxyInfoResponse>, StatusCode> {
    // 获取代理配置并立即释放锁
    let proxies = {
        let state = state.lock().await;
        state.proxies.clone()
    };

    let proxies_count = proxies.get_proxies().len();
    let general_proxy = proxies.get_general().to_string();

    Ok(Json(ProxyInfoResponse {
        status: ApiStatus::Success,
        proxies: Some(proxies),
        proxies_count,
        general_proxy: Some(general_proxy),
        message: None,
    }))
}

// 更新代理配置
pub async fn handle_set_proxies(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<ProxyUpdateRequest>,
) -> Result<Json<ProxyInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取新的代理配置
    let mut proxies = request.proxies;

    // 更新全局代理池并保存配置
    if let Err(e) = proxies.update_and_save().await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Owned(format!(
                    "Failed to save proxy configuration: {e}"
                ))),
                message: Some(Cow::Borrowed("无法保存代理配置")),
            }),
        ));
    }

    // 获取通用代理信息（在更新应用状态前）
    let proxies_count = proxies.get_proxies().len();

    // 只在需要更新应用状态时持有锁
    {
        let mut state_guard = state.lock().await;
        // 更新应用状态（完全覆盖）
        state_guard.proxies = proxies;
    }

    Ok(Json(ProxyInfoResponse {
        status: ApiStatus::Success,
        proxies: None,
        proxies_count,
        general_proxy: None,
        message: Some("代理配置已更新".to_string()),
    }))
}

// 添加新的代理
pub async fn handle_add_proxy(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<ProxyAddRequest>,
) -> Result<Json<ProxyInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取当前的代理配置
    let mut proxies = {
        let state_guard = state.lock().await;
        state_guard.proxies.clone()
    };

    // 创建现有代理名称的集合
    let existing_proxies: std::collections::HashSet<String> =
        proxies.get_proxies().keys().cloned().collect();

    // 处理新的代理
    let mut added_count = 0;

    for (name, proxy) in &request.proxies {
        // 跳过已存在的代理
        if existing_proxies.contains(name) {
            continue;
        }

        // 直接添加新的代理
        proxies.add_proxy(name.clone(), proxy.clone());
        added_count += 1;
    }

    // 如果有新代理才进行后续操作
    if added_count > 0 {
        // 更新全局代理池并保存配置
        if let Err(e) = proxies.update_and_save().await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    status: ApiStatus::Error,
                    code: None,
                    error: Some(Cow::Owned(format!(
                        "Failed to save proxy configuration: {e}"
                    ))),
                    message: Some(Cow::Borrowed("无法保存代理配置")),
                }),
            ));
        }

        // 获取更新后的信息
        let proxies_count = proxies.get_proxies().len();

        // 更新应用状态，只在需要时持有锁
        {
            let mut state_guard = state.lock().await;
            state_guard.proxies = proxies.clone();
        }

        Ok(Json(ProxyInfoResponse {
            status: ApiStatus::Success,
            proxies: None,
            proxies_count,
            general_proxy: None,
            message: Some(format!("已添加 {added_count} 个新代理")),
        }))
    } else {
        // 如果没有新代理，返回当前状态
        let general_proxy = proxies.get_general().to_string();
        let proxies_count = proxies.get_proxies().len();

        Ok(Json(ProxyInfoResponse {
            status: ApiStatus::Success,
            proxies: Some(proxies),
            proxies_count,
            general_proxy: Some(general_proxy),
            message: Some("没有添加新代理".to_string()),
        }))
    }
}

// 删除指定的代理
pub async fn handle_delete_proxies(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<ProxiesDeleteRequest>,
) -> Result<Json<ProxiesDeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取当前的代理配置并计算失败的代理名称
    let mut proxies = {
        let state_guard = state.lock().await;
        state_guard.proxies.clone()
    };

    // 计算失败的代理名称
    let failed_names: Vec<String> = request
        .names
        .iter()
        .filter(|name| !proxies.get_proxies().contains_key(*name))
        .cloned()
        .collect();

    // 删除指定的代理
    for name in &request.names {
        proxies.remove_proxy(name);
    }

    // 更新全局代理池并保存配置
    if let Err(e) = proxies.update_and_save().await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Owned(format!(
                    "Failed to save proxy configuration: {e}"
                ))),
                message: Some(Cow::Borrowed("无法保存代理配置")),
            }),
        ));
    }

    // 更新应用状态，只在需要时持有锁
    {
        let mut state_guard = state.lock().await;
        state_guard.proxies = proxies.clone();
    }

    // 根据expectation返回不同的结果
    let updated_proxies = if request.expectation.needs_updated_tokens() {
        Some(proxies)
    } else {
        None
    };

    Ok(Json(ProxiesDeleteResponse {
        status: ApiStatus::Success,
        updated_proxies,
        failed_names: if request.expectation.needs_failed_tokens() && !failed_names.is_empty() {
            Some(failed_names)
        } else {
            None
        },
    }))
}

// 设置通用代理
pub async fn handle_set_general_proxy(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<SetGeneralProxyRequest>,
) -> Result<Json<CommonResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 获取当前的代理配置
    let mut proxies = {
        let state_guard = state.lock().await;
        state_guard.proxies.clone()
    };

    // 检查代理名称是否存在
    if !proxies.get_proxies().contains_key(&request.name) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Borrowed("Proxy name not found")),
                message: Some(Cow::Borrowed("代理名称不存在")),
            }),
        ));
    }

    // 设置通用代理
    proxies.set_general(&request.name);

    // 更新全局代理池并保存配置
    if let Err(e) = proxies.update_and_save().await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                status: ApiStatus::Error,
                code: None,
                error: Some(Cow::Owned(format!(
                    "Failed to save proxy configuration: {e}"
                ))),
                message: Some(Cow::Borrowed("无法保存代理配置")),
            }),
        ));
    }

    // 更新应用状态，只在需要时持有锁
    {
        let mut state_guard = state.lock().await;
        state_guard.proxies = proxies;
    }

    Ok(Json(CommonResponse {
        status: ApiStatus::Success,
        message: Some("通用代理已设置".to_string()),
    }))
}
