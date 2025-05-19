macro_rules! def_header_name {
    ($($name:ident => $value:expr),+ $(,)?) => {
        $(paste::paste! {
            #[inline]
            pub(crate) fn [<header_name_ $name>]() -> &'static http::header::HeaderName {
                static HEADER_NAME: std::sync::OnceLock<http::header::HeaderName> = std::sync::OnceLock::new();
                HEADER_NAME.get_or_init(|| http::header::HeaderName::from_static($value))
            }
        })+
    };
}

macro_rules! def_header_value {
    ($($name:ident => $value:expr),+ $(,)?) => {
        $(paste::paste! {
            #[inline]
            pub fn [<header_value_ $name>]() -> &'static http::header::HeaderValue {
                static HEADER_NAME: std::sync::OnceLock<http::header::HeaderValue> = std::sync::OnceLock::new();
                HEADER_NAME.get_or_init(|| http::header::HeaderValue::from_static($value))
            }
        })+
    };
}

def_header_value!(
    one => "1",
    encoding => "gzip",
    encodings => "gzip,br",
    accept => "*/*",
    language => "en-US",
    empty => "empty",
    cors => "cors",
    no_cache => "no-cache",
    no_cache_revalidate => "no-cache, must-revalidate",
    ua_win => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    same_origin => "same-origin",
    keep_alive => "keep-alive",
    trailers => "trailers",
    u_eq_0 => "u=0",
    connect_es => "connect-es/1.6.1",
    not_a_brand => "\"Not-A.Brand\";v=\"99\", \"Chromium\";v=\"124\"",
    mobile_no => "?0",
    windows => "\"Windows\"",
    ua_cursor => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.42.5 Chrome/124.0.6367.243 Electron/30.4.0 Safari/537.36",
    vscode_origin => "vscode-file://vscode-app",
    cross_site => "cross-site",
    gzip_deflate => "gzip, deflate",
    event_stream => "text/event-stream",
    chunked => "chunked",
    json => "application/json",
    proto => "application/proto",
    connect_proto => "application/connect+proto",

    // Content type constants
    text_html_utf8 => "text/html;charset=utf-8",
    text_plain_utf8 => "text/plain;charset=utf-8",
    text_css_utf8 => "text/css;charset=utf-8",
    text_js_utf8 => "text/javascript;charset=utf-8"
);

def_header_name!(
    proxy_host => "x-co",
    connect_accept_encoding => "connect-accept-encoding",
    connect_protocol_version => "connect-protocol-version",
    ghost_mode => "x-ghost-mode",
    amzn_trace_id => "x-amzn-trace-id",
    client_key => "x-client-key",
    cursor_checksum => "x-cursor-checksum",
    cursor_client_version => "x-cursor-client-version",
    cursor_timezone => "x-cursor-timezone",
    request_id => "x-request-id",
    sec_ch_ua => "sec-ch-ua",
    sec_ch_ua_mobile => "sec-ch-ua-mobile",
    sec_ch_ua_platform => "sec-ch-ua-platform",
    sec_fetch_dest => "sec-fetch-dest",
    sec_fetch_mode => "sec-fetch-mode",
    sec_fetch_site => "sec-fetch-site",
    sec_gpc => "sec-gpc",
    priority => "priority",
);
