use crate::app::{
    constant::{
        CURSOR_API2_HOST, CURSOR_HOST, CURSOR_SETTINGS_URL, TRUE, header_name_amzn_trace_id,
        header_name_client_key, header_name_connect_accept_encoding,
        header_name_connect_protocol_version, header_name_cursor_checksum,
        header_name_cursor_client_version, header_name_cursor_timezone, header_name_ghost_mode,
        header_name_priority, header_name_proxy_host, header_name_request_id,
        header_name_sec_ch_ua, header_name_sec_ch_ua_mobile, header_name_sec_ch_ua_platform,
        header_name_sec_fetch_dest, header_name_sec_fetch_mode, header_name_sec_fetch_site,
        header_name_sec_gpc, header_value_accept, header_value_chunked, header_value_connect_es,
        header_value_connect_proto, header_value_cors, header_value_cross_site, header_value_empty,
        header_value_encoding, header_value_encodings, header_value_gzip_deflate,
        header_value_keep_alive, header_value_language, header_value_mobile_no,
        header_value_no_cache, header_value_not_a_brand, header_value_one, header_value_proto,
        header_value_same_origin, header_value_trailers, header_value_u_eq_0,
        header_value_ua_cursor, header_value_ua_win, header_value_vscode_origin,
        header_value_windows,
    },
    lazy::{
        PRI_REVERSE_PROXY_HOST, PUB_REVERSE_PROXY_HOST, USE_PRI_REVERSE_PROXY,
        USE_PUB_REVERSE_PROXY, cursor_api2_stripe_url, cursor_token_poll_url,
        cursor_token_upgrade_url, cursor_usage_api_url, cursor_user_api_url,
    },
};
use reqwest::{
    Client, Method, RequestBuilder,
    header::{
        ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, CONTENT_LENGTH,
        CONTENT_TYPE, COOKIE, DNT, HOST, ORIGIN, PRAGMA, REFERER, TE, TRANSFER_ENCODING,
        USER_AGENT,
    },
};

#[inline]
fn get_client_and_host<'a>(
    client: &Client,
    method: Method,
    url: &'a str,
    is_pri: bool,
    real_host: &'a str,
) -> (RequestBuilder, &'a str) {
    if is_pri && *USE_PRI_REVERSE_PROXY {
        (
            client
                .request(method, url)
                .header(header_name_proxy_host(), real_host),
            PRI_REVERSE_PROXY_HOST.as_str(),
        )
    } else if !is_pri && *USE_PUB_REVERSE_PROXY {
        (
            client
                .request(method, url)
                .header(header_name_proxy_host(), real_host),
            PUB_REVERSE_PROXY_HOST.as_str(),
        )
    } else {
        (client.request(method, url), real_host)
    }
}

pub(crate) struct AiServiceRequest<'a> {
    pub(crate) client: Client,
    pub(crate) auth_token: &'a str,
    pub(crate) checksum: &'a str,
    pub(crate) client_key: &'a str,
    pub(crate) url: &'a str,
    pub(crate) is_stream: bool,
    pub(crate) timezone: &'static str,
    pub(crate) trace_id: &'a str,
    pub(crate) is_pri: bool,
}

/// 返回预构建的 Cursor API 客户端
///
/// # 参数
///
/// * `auth_token` - 授权令牌
/// * `checksum` - 校验和
/// * `endpoint` - API 端点路径
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_request(req: AiServiceRequest) -> RequestBuilder {
    let (builder, host) = get_client_and_host(
        &req.client,
        Method::POST,
        req.url,
        req.is_pri,
        CURSOR_API2_HOST,
    );

    builder
        .header(
            CONTENT_TYPE,
            if req.is_stream {
                header_value_connect_proto()
            } else {
                header_value_proto()
            },
        )
        .bearer_auth(req.auth_token)
        .header(
            header_name_connect_accept_encoding(),
            header_value_encoding(),
        )
        .header(header_name_connect_protocol_version(), header_value_one())
        .header(USER_AGENT, header_value_connect_es())
        .header(
            header_name_amzn_trace_id(),
            format!("Root={}", req.trace_id),
        )
        .header(header_name_client_key(), req.client_key)
        .header(header_name_cursor_checksum(), req.checksum)
        .header(header_name_cursor_client_version(), "0.42.5")
        .header(header_name_cursor_timezone(), req.timezone)
        .header(header_name_ghost_mode(), TRUE)
        .header(header_name_request_id(), req.trace_id)
        .header(HOST, host)
        .header(CONNECTION, header_value_keep_alive())
        .header(TRANSFER_ENCODING, header_value_chunked())
}

/// 返回预构建的获取 Stripe 账户信息的 Cursor API 客户端
///
/// # 参数
///
/// * `auth_token` - 授权令牌
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_profile_request(client: &Client, auth_token: &str, is_pri: bool) -> RequestBuilder {
    let (builder, host) = get_client_and_host(
        client,
        Method::GET,
        cursor_api2_stripe_url(is_pri),
        is_pri,
        CURSOR_API2_HOST,
    );

    builder
        .header(HOST, host)
        .header(header_name_sec_ch_ua(), header_value_not_a_brand())
        .header(header_name_ghost_mode(), TRUE)
        .header(header_name_sec_ch_ua_mobile(), header_value_mobile_no())
        .bearer_auth(auth_token)
        .header(USER_AGENT, header_value_ua_cursor())
        .header(header_name_sec_ch_ua_platform(), header_value_windows())
        .header(ACCEPT, header_value_accept())
        .header(ORIGIN, header_value_vscode_origin())
        .header(header_name_sec_fetch_site(), header_value_cross_site())
        .header(header_name_sec_fetch_mode(), header_value_cors())
        .header(header_name_sec_fetch_dest(), header_value_empty())
        .header(ACCEPT_ENCODING, header_value_encodings())
        .header(ACCEPT_LANGUAGE, header_value_language())
        .header(header_name_priority(), header_value_u_eq_0())
}

/// 返回预构建的获取使用情况的 Cursor API 客户端
///
/// # 参数
///
/// * `user_id` - 用户 ID
/// * `auth_token` - 授权令牌
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_usage_request(
    client: &Client,
    user_id: &str,
    auth_token: &str,
    is_pri: bool,
) -> RequestBuilder {
    let (client, host) = get_client_and_host(
        client,
        Method::GET,
        cursor_usage_api_url(is_pri),
        is_pri,
        CURSOR_HOST,
    );

    client
        .header(HOST, host)
        .header(USER_AGENT, header_value_ua_win())
        .header(ACCEPT, header_value_accept())
        .header(ACCEPT_LANGUAGE, header_value_language())
        .header(ACCEPT_ENCODING, header_value_encodings())
        .header(REFERER, CURSOR_SETTINGS_URL)
        .header(DNT, header_value_one())
        .header(header_name_sec_gpc(), header_value_one())
        .header(header_name_sec_fetch_dest(), header_value_empty())
        .header(header_name_sec_fetch_mode(), header_value_cors())
        .header(header_name_sec_fetch_site(), header_value_same_origin())
        .header(CONNECTION, header_value_keep_alive())
        .header(PRAGMA, header_value_no_cache())
        .header(CACHE_CONTROL, header_value_no_cache())
        .header(TE, header_value_trailers())
        .header(header_name_priority(), header_value_u_eq_0())
        .header(
            COOKIE,
            format!("WorkosCursorSessionToken={user_id}%3A%3A{auth_token}"),
        )
        .query(&[("user", user_id)])
}

/// 返回预构建的获取用户信息的 Cursor API 客户端
///
/// # 参数
///
/// * `user_id` - 用户 ID
/// * `auth_token` - 授权令牌
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_userinfo_request(
    client: &Client,
    user_id: &str,
    auth_token: &str,
    is_pri: bool,
) -> RequestBuilder {
    let (client, host) = get_client_and_host(
        client,
        Method::GET,
        cursor_user_api_url(is_pri),
        is_pri,
        CURSOR_HOST,
    );

    client
        .header(HOST, host)
        .header(USER_AGENT, header_value_ua_win())
        .header(ACCEPT, header_value_accept())
        .header(ACCEPT_LANGUAGE, header_value_language())
        .header(ACCEPT_ENCODING, header_value_encodings())
        .header(REFERER, CURSOR_SETTINGS_URL)
        .header(DNT, header_value_one())
        .header(header_name_sec_gpc(), header_value_one())
        .header(header_name_sec_fetch_dest(), header_value_empty())
        .header(header_name_sec_fetch_mode(), header_value_cors())
        .header(header_name_sec_fetch_site(), header_value_same_origin())
        .header(CONNECTION, header_value_keep_alive())
        .header(PRAGMA, header_value_no_cache())
        .header(CACHE_CONTROL, header_value_no_cache())
        .header(TE, header_value_trailers())
        .header(header_name_priority(), header_value_u_eq_0())
        .header(
            COOKIE,
            format!("WorkosCursorSessionToken={user_id}%3A%3A{auth_token}"),
        )
}

pub fn build_token_upgrade_request(
    client: &Client,
    uuid: &str,
    challenge: &str,
    user_id: &str,
    auth_token: &str,
    is_pri: bool,
) -> RequestBuilder {
    let (client, host) = get_client_and_host(
        client,
        Method::POST,
        cursor_token_upgrade_url(is_pri),
        is_pri,
        CURSOR_HOST,
    );

    let body = format!("{{\"uuid\":\"{uuid}\",\"challenge\":\"{challenge}\"}}");

    client
        .header(HOST, host)
        .header(USER_AGENT, header_value_ua_win())
        .header(ACCEPT, header_value_accept())
        .header(ACCEPT_LANGUAGE, header_value_language())
        .header(ACCEPT_ENCODING, header_value_encodings())
        .header(
            REFERER,
            format!(
                "https://cursor.com/loginDeepControl?challenge={challenge}&uuid={uuid}&mode=login"
            ),
        )
        .header(CONTENT_TYPE, "application/json")
        .header(CONTENT_LENGTH, body.len())
        .header(DNT, header_value_one())
        .header(header_name_sec_gpc(), header_value_one())
        .header(header_name_sec_fetch_dest(), header_value_empty())
        .header(header_name_sec_fetch_mode(), header_value_cors())
        .header(header_name_sec_fetch_site(), header_value_same_origin())
        .header(CONNECTION, header_value_keep_alive())
        .header(PRAGMA, header_value_no_cache())
        .header(CACHE_CONTROL, header_value_no_cache())
        .header(TE, header_value_trailers())
        .header(header_name_priority(), header_value_u_eq_0())
        .header(
            COOKIE,
            format!("WorkosCursorSessionToken={user_id}%3A%3A{auth_token}"),
        )
        .body(body)
}

pub fn build_token_poll_request(
    client: &Client,
    uuid: &str,
    verifier: &str,
    is_pri: bool,
) -> RequestBuilder {
    let (client, host) = get_client_and_host(
        client,
        Method::GET,
        cursor_token_poll_url(is_pri),
        is_pri,
        CURSOR_API2_HOST,
    );
    client
        .header(HOST, host)
        .header(ACCEPT_ENCODING, header_value_gzip_deflate())
        .header(ACCEPT_LANGUAGE, header_value_language())
        .header(USER_AGENT, header_value_ua_cursor())
        .header(ORIGIN, header_value_vscode_origin())
        .header(header_name_ghost_mode(), TRUE)
        .header(ACCEPT, header_value_accept())
        .query(&[("uuid", uuid), ("verifier", verifier)])
}
