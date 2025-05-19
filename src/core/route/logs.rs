use crate::{
    app::{
        constant::{
            AUTHORIZATION_BEARER_PREFIX, ROUTE_LOGS_PATH, header_value_text_html_utf8,
            header_value_text_plain_utf8,
        },
        lazy::AUTH_TOKEN,
        model::{AppConfig, AppState, LogStatus, PageContent, RequestLog},
    },
    common::{
        model::{ApiStatus, userinfo::MembershipType},
        utils::extract_token,
    },
};
use axum::{
    Json,
    body::Body,
    extract::State,
    http::{
        HeaderMap, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Local};
use std::{str::FromStr as _, sync::Arc};
use tokio::sync::Mutex;

// 日志处理
pub async fn handle_logs() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_LOGS_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!("../../../static/logs.min.html")))
            .unwrap(),
        PageContent::Text(content) => Response::builder()
            .header(CONTENT_TYPE, header_value_text_plain_utf8())
            .body(Body::from(content))
            .unwrap(),
        PageContent::Html(content) => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(content))
            .unwrap(),
    }
}

#[derive(serde::Deserialize, Default)]
pub struct LogsQueryParams {
    pub limit: Option<usize>,               // 返回记录数量限制
    pub offset: Option<usize>,              // 起始位置偏移量
    pub status: Option<String>,             // 按状态过滤
    pub model: Option<String>,              // 按模型过滤
    pub from_date: Option<DateTime<Local>>, // 开始日期
    pub to_date: Option<DateTime<Local>>,   // 结束日期
    pub email: Option<String>,              // 按用户邮箱过滤
    pub membership_type: Option<String>,    // 按会员类型过滤 (free/free_trial/pro/enterprise)
    pub min_total_time: Option<f64>,        // 按最小总耗时过滤
    pub max_total_time: Option<f64>,        // 按最大总耗时过滤
    pub stream: Option<bool>,               // 按是否为流式请求过滤
    pub has_error: Option<bool>,            // 按是否有错误过滤
    pub has_chain: Option<bool>,            // 按是否有chain过滤
}

#[derive(serde::Deserialize)]
pub struct LogsRequest {
    #[serde(default)]
    pub query: LogsQueryParams,
}

pub async fn handle_logs_post(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<LogsRequest>,
) -> Result<Json<LogsResponse>, StatusCode> {
    let auth_token = AUTH_TOKEN.as_str();

    // 获取认证头
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let state = state.lock().await;

    // 如果状态存在但无效，直接返回空结果
    if let Some(status) = &request.query.status {
        if LogStatus::from_str_name(status).is_none() {
            return Ok(Json(LogsResponse {
                status: ApiStatus::Success,
                total: 0,
                active: None,
                error: None,
                logs: Vec::new(),
                timestamp: Local::now(),
            }));
        }
    }

    // 如果会员类型存在但无效，直接返回空结果
    let membership_enum = if let Some(membership_type) = &request.query.membership_type {
        match MembershipType::from_str(membership_type) {
            Ok(m) => Some(m),
            Err(_) => {
                return Ok(Json(LogsResponse {
                    status: ApiStatus::Success,
                    total: 0,
                    active: None,
                    error: None,
                    logs: Vec::new(),
                    timestamp: Local::now(),
                }));
            }
        }
    } else {
        None
    };

    // 准备日志数据（管理员或特定用户的）
    let mut iterator = Box::new(state.request_manager.request_logs.iter())
        as Box<dyn Iterator<Item = &RequestLog>>;
    if auth_header != auth_token {
        // 解析 token
        let token_part = extract_token(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;

        // 筛选符合条件的日志
        iterator = Box::new(iterator.filter(move |log| log.token_info.token == token_part));
    };

    // 按状态过滤
    if let Some(status) = &request.query.status {
        iterator = Box::new(iterator.filter(move |log| log.status.as_str_name() == status));
    }

    // 按模型过滤
    if let Some(model) = &request.query.model {
        iterator = Box::new(iterator.filter(move |log| log.model.contains(model)));
    }

    // 按用户邮箱过滤
    if let Some(email) = &request.query.email {
        iterator = Box::new(iterator.filter(move |log| {
            log.token_info
                .profile
                .as_ref()
                .map(|p| p.user.email.contains(email))
                .unwrap_or(false)
        }));
    }

    // 按会员类型过滤
    if let Some(membership_type) = membership_enum {
        iterator = Box::new(iterator.filter(move |log| {
            log.token_info
                .profile
                .as_ref()
                .map(|p| p.stripe.membership_type == membership_type)
                .unwrap_or(false)
        }));
    }

    // 按总耗时范围过滤
    if let Some(min_time) = request.query.min_total_time {
        iterator = Box::new(iterator.filter(move |log| log.timing.total >= min_time));
    }

    if let Some(max_time) = request.query.max_total_time {
        iterator = Box::new(iterator.filter(move |log| log.timing.total <= max_time));
    }

    // 按是否为流式请求过滤
    if let Some(stream) = request.query.stream {
        iterator = Box::new(iterator.filter(move |log| log.stream == stream));
    }

    // 按是否有错误过滤
    if let Some(has_error) = request.query.has_error {
        iterator = Box::new(iterator.filter(move |log| log.error.is_some() == has_error));
    }

    // 按是否有chain过滤
    if let Some(has_chain) = request.query.has_chain {
        iterator = Box::new(iterator.filter(move |log| log.chain.is_some() == has_chain));
    }

    // 按日期范围过滤
    if let Some(from_date) = request.query.from_date {
        iterator = Box::new(iterator.filter(move |log| log.timestamp >= from_date));
    }

    if let Some(to_date) = request.query.to_date {
        iterator = Box::new(iterator.filter(move |log| log.timestamp <= to_date));
    }

    // 获取总数
    let filtered_log_refs: Vec<_> = iterator.collect();
    let total = filtered_log_refs.len() as u64;

    // 应用分页
    let paginated_log_refs = filtered_log_refs
        .into_iter()
        .skip(request.query.offset.unwrap_or(0))
        .take(request.query.limit.unwrap_or(usize::MAX));

    let result_logs: Vec<RequestLog> = paginated_log_refs.cloned().collect();
    let active = if auth_header == auth_token {
        Some(state.request_manager.active_requests)
    } else {
        None
    };
    let error = if auth_header == auth_token {
        Some(state.request_manager.error_requests)
    } else {
        None
    };

    drop(state);

    Ok(Json(LogsResponse {
        status: ApiStatus::Success,
        total,
        active,
        error,
        logs: result_logs,
        timestamp: Local::now(),
    }))
}

#[derive(serde::Serialize)]
pub struct LogsResponse {
    pub status: ApiStatus,
    pub total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<u64>,
    pub logs: Vec<RequestLog>,
    pub timestamp: DateTime<Local>,
}
