use super::generate_checksum_with_repair;
use crate::app::{
    constant::{COMMA, EMPTY_STRING},
    lazy::TOKEN_LIST_FILE,
    model::TokenInfo,
};
use crate::common::model::token::TokenPayload;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Local, TimeZone};

// 规范化文件内容并写入
fn normalize_and_write(content: &str, file_path: &str) -> String {
    let normalized = content.replace("\r\n", "\n");
    if normalized != content {
        if let Err(e) = std::fs::write(file_path, &normalized) {
            eprintln!("警告: 无法更新规范化的文件: {}", e);
        }
    }
    normalized
}

// 解析token
pub fn parse_token(token_part: &str) -> String {
    // 查找最后一个:或%3A的位置
    let colon_pos = token_part.rfind(':');
    let encoded_colon_pos = token_part.rfind("%3A");

    match (colon_pos, encoded_colon_pos) {
        (None, None) => token_part.to_string(),
        (Some(pos1), None) => token_part[(pos1 + 1)..].to_string(),
        (None, Some(pos2)) => token_part[(pos2 + 3)..].to_string(),
        (Some(pos1), Some(pos2)) => {
            // 取较大的位置作为分隔点
            let pos = pos1.max(pos2);
            let start = if pos == pos2 { pos + 3 } else { pos + 1 };
            token_part[start..].to_string()
        }
    }
}

// Token 加载函数
pub fn load_tokens() -> Vec<TokenInfo> {
    let token_list_file = TOKEN_LIST_FILE.as_str();

    // 确保文件存在
    if !std::path::Path::new(&token_list_file).exists() {
        if let Err(e) = std::fs::write(&token_list_file, EMPTY_STRING) {
            eprintln!("警告: 无法创建文件 '{}': {}", &token_list_file, e);
        }
    }

    // 读取和规范化 token-list 文件
    let token_map: std::collections::HashMap<String, String> =
        match std::fs::read_to_string(&token_list_file) {
            Ok(content) => {
                let normalized = normalize_and_write(&content, &token_list_file);
                normalized
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            return None;
                        }

                        let parts: Vec<&str> = line.split(COMMA).collect();
                        match parts[..] {
                            [token_part, checksum] => {
                                let token = parse_token(token_part);
                                Some((token, generate_checksum_with_repair(checksum)))
                            }
                            _ => {
                                eprintln!("警告: 忽略无效的token-list行: {}", line);
                                None
                            }
                        }
                    })
                    .collect()
            }
            Err(e) => {
                eprintln!("警告: 无法读取token-list文件: {}", e);
                std::collections::HashMap::new()
            }
        };

    // 更新 token-list 文件
    let token_list_content = token_map
        .iter()
        .map(|(token, checksum)| format!("{},{}", token, checksum))
        .collect::<Vec<_>>()
        .join("\n");

    if let Err(e) = std::fs::write(&token_list_file, token_list_content) {
        eprintln!("警告: 无法更新token-list文件: {}", e);
    }

    // 转换为 TokenInfo vector
    token_map
        .into_iter()
        .map(|(token, checksum)| TokenInfo {
            token: token.clone(),
            checksum,
            profile: None,
        })
        .collect()
}

pub fn write_tokens(token_infos: &[TokenInfo], file_path: &str) -> std::io::Result<()> {
    let content = token_infos
        .iter()
        .map(|info| format!("{},{}", info.token, info.checksum))
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write(file_path, content)
}

pub(super) const HEADER_B64: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
pub(super) const ISSUER: &str = "https://authentication.cursor.sh";
pub(super) const SCOPE: &str = "openid profile email offline_access";
pub(super) const AUDIENCE: &str = "https://cursor.com";

// 验证jwt token是否有效
pub fn validate_token(token: &str) -> bool {
    // 检查 token 格式
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    if parts[0] != HEADER_B64 {
        return false;
    }

    // 解码 payload
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return false,
    };

    // 转换为字符串
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // 解析为 TokenPayload
    let payload: TokenPayload = match serde_json::from_str(&payload_str) {
        Ok(p) => p,
        Err(_) => return false,
    };

    // 验证 time 字段
    if let Ok(time_value) = payload.time.parse::<i64>() {
        let current_time = chrono::Utc::now().timestamp();
        if time_value > current_time {
            return false;
        }
    } else {
        return false;
    }

    // 验证 randomness 格式
    let bytes = payload.randomness.as_bytes();
    if bytes.len() != 18 {
        return false;
    }

    // 单次遍历完成所有字符校验
    for (i, &b) in bytes.iter().enumerate() {
        let valid = match i {
            // 16进制数字部分
            0..=7 | 9..=12 | 14..=17 => b.is_ascii_hexdigit(),
            // 连字符部分
            8 | 13 => b == b'-',
            _ => unreachable!(),
        };

        if !valid {
            return false;
        }
    }

    // 验证过期时间
    let current_time = chrono::Utc::now().timestamp();
    if current_time > payload.exp {
        return false;
    }

    // 验证发行者
    if payload.iss != ISSUER {
        return false;
    }

    // 验证授权范围
    if payload.scope != SCOPE {
        return false;
    }

    // 验证受众
    if payload.aud != AUDIENCE {
        return false;
    }

    true
}

// 从 JWT token 中提取用户 ID
pub fn extract_user_id(token: &str) -> Option<String> {
    // JWT token 由3部分组成，用 . 分隔
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // 解码 payload (第二部分)
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return None,
    };

    // 将 payload 转换为字符串
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 解析为 TokenPayload
    let payload: TokenPayload = match serde_json::from_str(&payload_str) {
        Ok(p) => p,
        Err(_) => return None,
    };

    // 提取 sub 字段
    Some(
        payload
            .sub
            .split('|')
            .nth(1)
            .unwrap_or(&payload.sub)
            .to_string(),
    )
}

// 从 JWT token 中提取 time 字段
pub fn extract_time(token: &str) -> Option<DateTime<Local>> {
    // JWT token 由3部分组成，用 . 分隔
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // 解码 payload (第二部分)
    let payload = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return None,
    };

    // 将 payload 转换为字符串
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 解析为 TokenPayload
    let payload: TokenPayload = match serde_json::from_str(&payload_str) {
        Ok(p) => p,
        Err(_) => return None,
    };

    // 提取时间戳并转换为本地时间
    payload
        .time
        .parse::<i64>()
        .ok()
        .and_then(|timestamp| Local.timestamp_opt(timestamp, 0).single())
}
