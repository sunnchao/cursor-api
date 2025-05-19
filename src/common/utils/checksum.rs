use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use sha2::{Digest, Sha256};

#[inline]
pub fn generate_hash() -> String {
    use rand::Rng as _;
    let mut v = rand::rng().random::<[u8; 32]>();
    if *crate::app::lazy::SAFE_HASH {
        v = Sha256::new().chain_update(v).finalize().into();
    }
    hex::encode(v)
}

#[inline]
fn obfuscate_bytes(bytes: &mut [u8]) {
    let mut prev: u8 = 165;
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let old_value = *byte;
        *byte = (old_value ^ prev).wrapping_add((idx % 256) as u8);
        prev = *byte;
    }
}

#[inline]
fn deobfuscate_bytes(bytes: &mut [u8]) {
    let mut prev: u8 = 165;
    for (idx, byte) in bytes.iter_mut().enumerate() {
        let temp = *byte;
        *byte = (*byte).wrapping_sub((idx % 256) as u8) ^ prev;
        prev = temp;
    }
}

pub fn generate_timestamp_header() -> [u8; 8] {
    static CACHE: std::sync::LazyLock<parking_lot::Mutex<(u64, [u8; 8])>> =
        std::sync::LazyLock::new(|| parking_lot::Mutex::new((0, [0u8; 8])));
    let timestamp = super::now_secs() / 1_000;

    let mut guard = CACHE.lock();
    if guard.0 == timestamp {
        return guard.1;
    } else {
        guard.0 = timestamp;
    }

    let mut timestamp_bytes = [
        ((timestamp >> 8) & 0xFF) as u8,
        (0xFF & timestamp) as u8,
        ((timestamp >> 24) & 0xFF) as u8,
        ((timestamp >> 16) & 0xFF) as u8,
        ((timestamp >> 8) & 0xFF) as u8,
        (0xFF & timestamp) as u8,
    ];

    obfuscate_bytes(&mut timestamp_bytes);
    let mut result = [0u8; 8];
    let _ = BASE64.encode_slice(&timestamp_bytes, &mut result);
    guard.1 = result;
    result
}

#[inline]
pub fn generate_checksum(device_id: &str, mac_addr: Option<&str>) -> String {
    let timestamp_header = generate_timestamp_header();
    let encoded = unsafe { str::from_utf8_unchecked(&timestamp_header) };
    match mac_addr {
        Some(mac) => format!("{encoded}{device_id}/{mac}"),
        None => format!("{encoded}{device_id}"),
    }
}

#[inline]
pub fn generate_checksum_with_default() -> String {
    generate_checksum(&generate_hash(), Some(&generate_hash()))
}

pub fn generate_checksum_with_repair(checksum: &str) -> String {
    let bytes = checksum.as_bytes();
    let len = bytes.len();

    // 长度快速检查
    if len != 72 && len != 129 && len != 137 {
        return generate_checksum_with_default();
    }

    // 单次遍历完成所有字符校验
    for (i, &b) in bytes.iter().enumerate() {
        let valid = match (len, i) {
            // 通用字符校验（排除非法字符）
            (_, _) if !b.is_ascii_alphanumeric() && b != b'/' && b != b'-' && b != b'_' => false,

            // 72字节格式：时间戳(8) + 设备哈希(64)
            (72, 8..=71) => b.is_ascii_hexdigit(),

            // 129字节格式：设备哈希(64) + '/' + MAC哈希(64)
            (129, 0..=63) => b.is_ascii_hexdigit(),
            (129, 64) => b == b'/',
            (129, 65..=128) => b.is_ascii_hexdigit(),

            // 137字节格式：时间戳(8) + 设备哈希(64) + '/' + MAC哈希(64)
            (137, 8..=71) => b.is_ascii_hexdigit(),
            (137, 72) => b == b'/',
            (137, 73..=136) => b.is_ascii_hexdigit(),

            // 时间戳部分不需要校验
            (72 | 137, 0..=7) => true,

            _ => unreachable!(),
        };

        if !valid {
            return generate_checksum_with_default();
        }
    }

    let timestamp_header = generate_timestamp_header();
    let encoded = unsafe { str::from_utf8_unchecked(&timestamp_header) };

    // 校验通过后构造结果
    match len {
        72 => format!(
            "{encoded}{}/{}",
            unsafe { std::str::from_utf8_unchecked(bytes.get_unchecked(8..)) },
            generate_hash()
        ),
        129 => format!(
            "{encoded}{}/{}",
            unsafe { std::str::from_utf8_unchecked(bytes.get_unchecked(..64)) },
            unsafe { std::str::from_utf8_unchecked(bytes.get_unchecked(65..)) }
        ),
        137 => format!(
            "{encoded}{}/{}",
            unsafe { std::str::from_utf8_unchecked(bytes.get_unchecked(8..72)) },
            unsafe { std::str::from_utf8_unchecked(bytes.get_unchecked(73..)) }
        ),
        _ => unreachable!(),
    }
}

pub fn extract_time_ks(timestamp_base64: &str) -> Option<u64> {
    let mut timestamp_bytes = BASE64.decode(timestamp_base64).ok()?;

    if timestamp_bytes.len() != 6 {
        return None;
    }

    deobfuscate_bytes(&mut timestamp_bytes);

    unsafe {
        if timestamp_bytes.get_unchecked(0) != timestamp_bytes.get_unchecked(4)
            || timestamp_bytes.get_unchecked(1) != timestamp_bytes.get_unchecked(5)
        {
            return None;
        }

        // 使用后四位还原 timestamp
        Some(
            ((*timestamp_bytes.get_unchecked(2) as u64) << 24)
                | ((*timestamp_bytes.get_unchecked(3) as u64) << 16)
                | ((*timestamp_bytes.get_unchecked(4) as u64) << 8)
                | (*timestamp_bytes.get_unchecked(5) as u64),
        )
    }
}

pub fn validate_checksum(checksum: &str) -> bool {
    let bytes = checksum.as_bytes();
    let len = bytes.len();

    // 长度门控
    if len != 72 && len != 137 {
        return false;
    }

    // 单次遍历完成所有字符校验
    for (i, &b) in bytes.iter().enumerate() {
        let valid = match (len, i) {
            // 通用字符校验（排除非法字符）
            (_, _) if !b.is_ascii_alphanumeric() && b != b'/' && b != b'-' && b != b'_' => false,

            // 格式校验
            (72, 0..=7) => true, // 时间戳部分（由extract_time_ks验证）
            (72, 8..=71) => b.is_ascii_hexdigit(),

            (137, 0..=7) => true,                     // 时间戳
            (137, 8..=71) => b.is_ascii_hexdigit(),   // 设备哈希
            (137, 72) => b == b'/',                   // 分割符（索引72是第73个字符）
            (137, 73..=136) => b.is_ascii_hexdigit(), // MAC哈希

            _ => unreachable!(),
        };

        if !valid {
            return false;
        }
    }

    // 统一时间戳验证（无需分层）
    let time_valid = extract_time_ks(unsafe { checksum.get_unchecked(..8) }).is_some();

    time_valid
}

/// 从校验通过的checksum中提取哈希值（需先通过validate_checksum验证）
/// 返回 (device_hash, mac_hash) ，mac_hash可能为空Vec
pub fn extract_hashes(checksum: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    // 前置条件：必须通过校验（确保长度和格式正确）
    if !validate_checksum(checksum) {
        return None;
    }

    // 根据长度直接切割，无需字符级验证（validate_checksum已保证）
    match checksum.len() {
        72 => {
            // 格式：8字节时间戳 + 64字节设备哈希
            let device_hash = hex::decode(unsafe { checksum.get_unchecked(8..) }).ok()?; // 8..72
            Some((device_hash, Vec::new()))
        }
        137 => {
            // 格式：8时间戳 + 64设备哈希 + '/' + 64MAC哈希
            // 直接按固定位置切割（validate_checksum已确保索引72是'/'）
            let device_hash = hex::decode(unsafe { checksum.get_unchecked(8..72) }).ok()?;
            let mac_hash = hex::decode(unsafe { checksum.get_unchecked(73..) }).ok()?; // 73..137
            Some((device_hash, mac_hash))
        }
        // validate_checksum已过滤其他长度，此处应为不可达代码
        _ => unreachable!("Invalid length after validation: {}", checksum.len()),
    }
}
