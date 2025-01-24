use crate::{
    chat::constant::ERR_NODATA,
    common::{model::userinfo::GetUserInfo, utils::{extract_token, get_token_profile}},
};
use axum::Json;

use super::tokens::TokenRequest;

pub async fn handle_user_info(Json(request): Json<TokenRequest>) -> Json<GetUserInfo> {
    let auth_token = match request.token {
        Some(token) => token,
        None => {
            return Json(GetUserInfo::Error {
                error: ERR_NODATA.to_string(),
            })
        }
    };

    let token = match extract_token(&auth_token) {
        Some(token) => token,
        None => {
            return Json(GetUserInfo::Error {
                error: ERR_NODATA.to_string(),
            })
        }
    };

    match get_token_profile(&token).await {
        Some(usage) => Json(GetUserInfo::Usage(usage)),
        None => Json(GetUserInfo::Error {
            error: ERR_NODATA.to_string(),
        }),
    }
}
