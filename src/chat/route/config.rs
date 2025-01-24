use crate::{
    app::{
        constant::{
            AUTHORIZATION_BEARER_PREFIX, CONTENT_TYPE_TEXT_CSS_WITH_UTF8, CONTENT_TYPE_TEXT_HTML_WITH_UTF8, CONTENT_TYPE_TEXT_JS_WITH_UTF8, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, ROUTE_ABOUT_PATH, ROUTE_BUILD_KEY_PATH, ROUTE_CONFIG_PATH, ROUTE_README_PATH, ROUTE_SHARED_JS_PATH, ROUTE_SHARED_STYLES_PATH
        },
        lazy::{AUTH_TOKEN, KEY_PREFIX},
        model::{AppConfig, BuildKeyRequest, BuildKeyResponse, PageContent, UsageCheckModelType},
    },
    chat::config::{key_config, KeyConfig},
    common::utils::{to_base64, token_to_tokeninfo},
};
use axum::{
    body::Body,
    extract::Path,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE, LOCATION},
        HeaderMap, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use prost::Message as _;

pub async fn handle_env_example() -> impl IntoResponse {
    Response::builder()
        .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8)
        .body(include_str!("../../../.env.example").to_string())
        .unwrap()
}

// 配置页面处理函数
pub async fn handle_config_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_CONFIG_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/config.min.html").to_string())
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

pub async fn handle_static(Path(path): Path<String>) -> impl IntoResponse {
    match path.as_str() {
        "shared-styles.css" => {
            match AppConfig::get_page_content(ROUTE_SHARED_STYLES_PATH).unwrap_or_default() {
                PageContent::Default => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_CSS_WITH_UTF8)
                    .body(include_str!("../../../static/shared-styles.min.css").to_string())
                    .unwrap(),
                PageContent::Text(content) | PageContent::Html(content) => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_CSS_WITH_UTF8)
                    .body(content.clone())
                    .unwrap(),
            }
        }
        "shared.js" => {
            match AppConfig::get_page_content(ROUTE_SHARED_JS_PATH).unwrap_or_default() {
                PageContent::Default => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_JS_WITH_UTF8)
                    .body(include_str!("../../../static/shared.min.js").to_string())
                    .unwrap(),
                PageContent::Text(content) | PageContent::Html(content) => Response::builder()
                    .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_JS_WITH_UTF8)
                    .body(content.clone())
                    .unwrap(),
            }
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not found".to_string())
            .unwrap(),
    }
}

pub async fn handle_readme() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_README_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/readme.min.html").to_string())
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

pub async fn handle_about() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_ABOUT_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, ROUTE_README_PATH)
            .body(Body::empty())
            .unwrap(),
        PageContent::Text(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8)
            .body(Body::from(content.clone()))
            .unwrap(),
        PageContent::Html(content) => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(Body::from(content.clone()))
            .unwrap(),
    }
}

pub async fn handle_build_key_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_BUILD_KEY_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, CONTENT_TYPE_TEXT_HTML_WITH_UTF8)
            .body(include_str!("../../../static/build_key.min.html").to_string())
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

        if auth_header.map_or(true, |h| h != AppConfig::get_share_token().as_str() && h != AUTH_TOKEN.as_str()) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(BuildKeyResponse::Error("Unauthorized".to_owned())),
            );
        }
    }

    // 验证并解析 auth_token
    let token_info = match token_to_tokeninfo(&request.auth_token) {
        Some(info) => info,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(BuildKeyResponse::Error("Invalid auth token".to_owned())),
            )
        }
    };

    // 构建 proto 消息
    let mut key_config = KeyConfig {
        auth_token: Some(token_info),
        enable_stream_check: request.enable_stream_check,
        include_stop_stream: request.include_stop_stream,
        disable_vision: request.disable_vision,
        enable_slow_pool: request.enable_slow_pool,
        usage_check_models: None,
    };

    if let Some(usage_check_models) = request.usage_check_models {
        let usage_check = key_config::UsageCheckModel {
            r#type: match usage_check_models.model_type {
                UsageCheckModelType::Default => {
                    key_config::usage_check_model::Type::Default as i32
                }
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
