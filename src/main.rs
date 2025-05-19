mod app;
mod common;
mod core;
mod leak;

use app::{
    config::handle_config_update,
    constant::{
        PKG_VERSION, ROUTE_ABOUT_PATH, ROUTE_API_PATH, ROUTE_BASIC_CALIBRATION_PATH,
        ROUTE_BUILD_KEY_PATH, ROUTE_CONFIG_PATH, ROUTE_ENV_EXAMPLE_PATH, ROUTE_GET_CHECKSUM,
        ROUTE_GET_HASH, ROUTE_GET_TIMESTAMP_HEADER, ROUTE_HEALTH_PATH, ROUTE_LOGS_PATH,
        ROUTE_PROXIES_ADD_PATH, ROUTE_PROXIES_DELETE_PATH, ROUTE_PROXIES_GET_PATH,
        ROUTE_PROXIES_PATH, ROUTE_PROXIES_SET_GENERAL_PATH, ROUTE_PROXIES_SET_PATH,
        ROUTE_README_PATH, ROUTE_ROOT_PATH, ROUTE_STATIC_PATH, ROUTE_TOKEN_UPGRADE_PATH,
        ROUTE_TOKENS_ADD_PATH, ROUTE_TOKENS_BY_TAG_GET_PATH, ROUTE_TOKENS_DELETE_PATH,
        ROUTE_TOKENS_GET_PATH, ROUTE_TOKENS_PATH, ROUTE_TOKENS_PROFILE_UPDATE_PATH,
        ROUTE_TOKENS_SET_PATH, ROUTE_TOKENS_STATUS_SET_PATH, ROUTE_TOKENS_TAGS_GET_PATH,
        ROUTE_TOKENS_TAGS_SET_PATH, ROUTE_TOKENS_UPGRADE_PATH, ROUTE_USER_INFO_PATH,
    },
    lazy::{AUTH_TOKEN, ROUTE_CHAT_PATH, ROUTE_MODELS_PATH},
    model::*,
};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use common::utils::{parse_string_from_env, parse_usize_from_env};
use core::{
    middleware::admin_auth_middleware,
    route::{
        handle_about, handle_add_proxy, handle_add_tokens, handle_api_page,
        handle_basic_calibration, handle_build_key, handle_build_key_page, handle_config_page,
        handle_delete_proxies, handle_delete_tokens, handle_env_example, handle_get_checksum,
        handle_get_hash, handle_get_proxies, handle_get_timestamp_header, handle_get_token_tags,
        handle_get_tokens, handle_get_tokens_by_tag, handle_health, handle_logs, handle_logs_post,
        handle_proxies_page, handle_readme, handle_root, handle_set_general_proxy,
        handle_set_proxies, handle_set_token_tags, handle_set_tokens, handle_set_tokens_status,
        handle_static, handle_token_upgrade, handle_tokens_page, handle_update_tokens_profile,
        handle_upgrade_tokens, handle_user_info,
    },
    service::{handle_chat, handle_models},
};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

#[tokio::main]
async fn main() {
    // 设置自定义 panic hook
    std::panic::set_hook(Box::new(|info| {
        // std::env::set_var("RUST_BACKTRACE", "1");
        if let Some(msg) = info.payload().downcast_ref::<String>() {
            eprintln!("{msg}");
        } else if let Some(msg) = info.payload().downcast_ref::<&str>() {
            eprintln!("{msg}");
        }
    }));

    // 加载环境变量
    dotenvy::dotenv().ok();

    if AUTH_TOKEN.is_empty() {
        panic!("AUTH_TOKEN must be set")
    };

    // 初始化全局配置
    AppConfig::init();

    // 初始化应用状态
    let state = Arc::new(Mutex::new(AppState::new().await));

    // 尝试加载保存的配置
    if let Err(e) = AppConfig::load_saved_config() {
        eprintln!("加载保存的配置失败: {e}");
    }

    // 创建一个克隆用于后台任务
    let state_for_reload = state.clone();

    // 启动后台任务在每个整1000秒时更新 checksum
    tokio::spawn(async move {
        loop {
            // 获取当前时间戳
            let now = common::utils::now_secs();

            // 计算距离下一个整1000秒的等待时间
            let next_reload = (now / 1000 + 1) * 1000;
            let wait_duration = next_reload - now;

            // 等待到下一个整1000秒
            tokio::time::sleep(std::time::Duration::from_secs(wait_duration)).await;

            let mut app_state = state_for_reload.lock().await;
            app_state.token_manager.update_checksum();
            // debug_println!("checksum 自动刷新: {next_reload}");
        }
    });

    // 创建一个克隆用于信号处理
    let state_for_shutdown = state.clone();

    // 设置关闭信号处理
    let shutdown_signal = async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        println!("正在关闭服务器...");

        // 保存配置
        if let Err(e) = AppConfig::save_config() {
            eprintln!("保存配置失败: {e}");
        } else {
            println!("配置已保存");
        }

        // 保存状态
        let state = state_for_shutdown.lock().await;
        if let Err(e) = state.save_state().await {
            eprintln!("保存状态失败: {e}");
        } else {
            println!("状态已保存");
        }
    };

    // 设置路由
    let app = Router::new()
        .route(ROUTE_ROOT_PATH, get(handle_root))
        .route(ROUTE_HEALTH_PATH, get(handle_health))
        .route(ROUTE_TOKENS_PATH, get(handle_tokens_page))
        .route(ROUTE_PROXIES_PATH, get(handle_proxies_page))
        .merge(
            Router::new()
                .route(ROUTE_TOKENS_GET_PATH, post(handle_get_tokens))
                .route(ROUTE_TOKENS_SET_PATH, post(handle_set_tokens))
                .route(ROUTE_TOKENS_ADD_PATH, post(handle_add_tokens))
                .route(ROUTE_TOKENS_DELETE_PATH, post(handle_delete_tokens))
                .route(ROUTE_TOKENS_TAGS_GET_PATH, post(handle_get_token_tags))
                .route(ROUTE_TOKENS_TAGS_SET_PATH, post(handle_set_token_tags))
                .route(ROUTE_TOKENS_BY_TAG_GET_PATH, post(handle_get_tokens_by_tag))
                .route(
                    ROUTE_TOKENS_PROFILE_UPDATE_PATH,
                    post(handle_update_tokens_profile),
                )
                .route(ROUTE_TOKENS_UPGRADE_PATH, post(handle_upgrade_tokens))
                .route(ROUTE_TOKENS_STATUS_SET_PATH, post(handle_set_tokens_status))
                .route(ROUTE_PROXIES_GET_PATH, post(handle_get_proxies))
                .route(ROUTE_PROXIES_SET_PATH, post(handle_set_proxies))
                .route(ROUTE_PROXIES_ADD_PATH, post(handle_add_proxy))
                .route(ROUTE_PROXIES_DELETE_PATH, post(handle_delete_proxies))
                .route(
                    ROUTE_PROXIES_SET_GENERAL_PATH,
                    post(handle_set_general_proxy),
                )
                .layer(middleware::from_fn(admin_auth_middleware)),
        )
        .route(ROUTE_MODELS_PATH.as_str(), get(handle_models))
        .route(ROUTE_CHAT_PATH.as_str(), post(handle_chat))
        // .route(ROUTE_MESSAGES_PATH.as_str(), post(handle_chat))
        .route(ROUTE_LOGS_PATH, get(handle_logs))
        .route(ROUTE_LOGS_PATH, post(handle_logs_post))
        .route(ROUTE_ENV_EXAMPLE_PATH, get(handle_env_example))
        .route(ROUTE_CONFIG_PATH, get(handle_config_page))
        .route(ROUTE_CONFIG_PATH, post(handle_config_update))
        .route(ROUTE_STATIC_PATH, get(handle_static))
        .route(ROUTE_ABOUT_PATH, get(handle_about))
        .route(ROUTE_README_PATH, get(handle_readme))
        .route(ROUTE_API_PATH, get(handle_api_page))
        .route(ROUTE_GET_HASH, get(handle_get_hash))
        .route(ROUTE_GET_CHECKSUM, get(handle_get_checksum))
        .route(ROUTE_GET_TIMESTAMP_HEADER, get(handle_get_timestamp_header))
        .route(ROUTE_BASIC_CALIBRATION_PATH, post(handle_basic_calibration))
        .route(ROUTE_USER_INFO_PATH, post(handle_user_info))
        .route(ROUTE_BUILD_KEY_PATH, get(handle_build_key_page))
        .route(ROUTE_BUILD_KEY_PATH, post(handle_build_key))
        .route(ROUTE_TOKEN_UPGRADE_PATH, post(handle_token_upgrade))
        .layer(RequestBodyLimitLayer::new(
            1024 * 1024 * parse_usize_from_env("REQUEST_BODY_LIMIT_MB", 2),
        ))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动服务器
    let port = parse_string_from_env("PORT", "3000");
    let addr = format!("0.0.0.0:{port}");
    println!("服务器运行在端口 {port}");
    #[cfg(not(feature = "__preview"))]
    println!("当前版本: v{PKG_VERSION}");
    #[cfg(feature = "__preview")]
    {
        const BUILD_VERSION: &str = include_str!("../VERSION");
        println!("当前版本: v{PKG_VERSION}+build.{BUILD_VERSION}");
    }
    #[cfg(feature = "__preview")]
    println!("当前是测试版，有问题及时反馈哦~");

    app::lazy::get_start_time();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("无法绑定到地址 {addr}: {e}");
            std::process::exit(1);
        });
    let server = axum::serve(listener, app);
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                eprintln!("服务器错误: {e}");
            }
        }
        _ = shutdown_signal => {
            println!("服务器已关闭");
        }
    }
}
