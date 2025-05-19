use crate::app::{
    constant::{
        ROUTE_ABOUT_PATH, ROUTE_API_PATH, ROUTE_BUILD_KEY_PATH, ROUTE_CONFIG_PATH,
        ROUTE_PROXIES_PATH, ROUTE_README_PATH, ROUTE_SHARED_JS_PATH, ROUTE_SHARED_STYLES_PATH,
        ROUTE_TOKENS_PATH, header_value_text_css_utf8, header_value_text_html_utf8,
        header_value_text_js_utf8, header_value_text_plain_utf8,
    },
    model::{AppConfig, PageContent},
};
use axum::{
    body::Body,
    extract::Path,
    http::{
        StatusCode,
        header::{CONTENT_TYPE, LOCATION},
    },
    response::{IntoResponse, Response},
};

pub async fn handle_env_example() -> impl IntoResponse {
    Response::builder()
        .header(CONTENT_TYPE, header_value_text_plain_utf8())
        .body(Body::from(include_str!("../../../.env.example")))
        .unwrap()
}

// 配置页面处理函数
pub async fn handle_config_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_CONFIG_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!("../../../static/config.min.html")))
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

pub async fn handle_static(Path(path): Path<String>) -> impl IntoResponse {
    match path.as_str() {
        "shared-styles.css" => {
            match AppConfig::get_page_content(ROUTE_SHARED_STYLES_PATH).unwrap_or_default() {
                PageContent::Default => Response::builder()
                    .header(CONTENT_TYPE, header_value_text_css_utf8())
                    .body(Body::from(include_str!(
                        "../../../static/shared-styles.min.css"
                    )))
                    .unwrap(),
                PageContent::Text(content) | PageContent::Html(content) => Response::builder()
                    .header(CONTENT_TYPE, header_value_text_css_utf8())
                    .body(Body::from(content))
                    .unwrap(),
            }
        }
        "shared.js" => {
            match AppConfig::get_page_content(ROUTE_SHARED_JS_PATH).unwrap_or_default() {
                PageContent::Default => Response::builder()
                    .header(CONTENT_TYPE, header_value_text_js_utf8())
                    .body(Body::from(
                        include_str!("../../../static/shared.min.js").to_string(),
                    ))
                    .unwrap(),
                PageContent::Text(content) | PageContent::Html(content) => Response::builder()
                    .header(CONTENT_TYPE, header_value_text_js_utf8())
                    .body(Body::from(content))
                    .unwrap(),
            }
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    }
}

pub async fn handle_readme() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_README_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!("../../../static/readme.min.html")))
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

pub async fn handle_about() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_ABOUT_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, ROUTE_README_PATH)
            .body(Body::empty())
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

pub async fn handle_build_key_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_BUILD_KEY_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!(
                "../../../static/build_key.min.html"
            )))
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

pub async fn handle_tokens_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_TOKENS_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!("../../../static/tokens.min.html")))
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

pub async fn handle_proxies_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_PROXIES_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!("../../../static/proxies.min.html")))
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

pub async fn handle_api_page() -> impl IntoResponse {
    match AppConfig::get_page_content(ROUTE_API_PATH).unwrap_or_default() {
        PageContent::Default => Response::builder()
            .header(CONTENT_TYPE, header_value_text_html_utf8())
            .body(Body::from(include_str!("../../../static/api.min.html")))
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
