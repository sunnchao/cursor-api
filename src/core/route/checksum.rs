use axum::{
    extract::Query,
    http::{HeaderMap, header::CONTENT_TYPE},
    response::{IntoResponse as _, Response},
};
use serde::Deserialize;

use crate::{
    app::constant::header_value_text_plain_utf8,
    common::utils::{
        generate_checksum_with_default, generate_checksum_with_repair, generate_hash,
        generate_timestamp_header,
    },
};

pub async fn handle_get_hash() -> Response {
    let hash = generate_hash();

    let headers = HeaderMap::from_iter([(CONTENT_TYPE, header_value_text_plain_utf8().clone())]);

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

    let headers = HeaderMap::from_iter([(CONTENT_TYPE, header_value_text_plain_utf8().clone())]);

    (headers, checksum).into_response()
}

pub async fn handle_get_timestamp_header() -> Response {
    let timestamp_header = generate_timestamp_header();

    let headers = HeaderMap::from_iter([(CONTENT_TYPE, header_value_text_plain_utf8().clone())]);

    (headers, timestamp_header).into_response()
}
