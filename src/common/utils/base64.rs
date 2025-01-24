// Base64 字符集 (a-z, A-Z, 0-9, -, _)
const BASE64_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";

// 预计算的 Base64 查找表，用于快速解码
const BASE64_LOOKUP: [i8; 256] = {
    let mut lookup = [-1i8; 256];
    let mut i = 0;
    while i < BASE64_CHARS.len() {
        lookup[BASE64_CHARS[i] as usize] = i as i8;
        i += 1;
    }
    lookup
};

/// 将字节切片编码为 Base64 字符串。
///
/// # Arguments
///
/// * `bytes`: 要编码的字节切片
///
/// # Returns
///
/// 编码后的 Base64 字符串
pub fn to_base64(bytes: &[u8]) -> String {
    // 预分配足够容量，避免多次分配内存
    let capacity = (bytes.len() + 2) / 3 * 4;
    let mut result = Vec::with_capacity(capacity);

    // 每三个字节为一组进行处理
    for chunk in bytes.chunks(3) {
        // 将三个字节合并为一个 u32
        let b1 = chunk[0] as u32;
        let b2 = chunk.get(1).map_or(0, |&b| b as u32);
        let b3 = chunk.get(2).map_or(0, |&b| b as u32);

        let n = (b1 << 16) | (b2 << 8) | b3;

        // 将 u32 拆分成四个 6 位的值，并根据查找表转换为 Base64 字符
        result.push(BASE64_CHARS[(n >> 18) as usize]);
        result.push(BASE64_CHARS[((n >> 12) & 0x3F) as usize]);

        // 如果 chunk 长度大于 1，则需要处理第二个字符
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((n >> 6) & 0x3F) as usize]);
            // 如果 chunk 长度大于 2，则需要处理第三个字符
            if chunk.len() > 2 {
                result.push(BASE64_CHARS[(n & 0x3F) as usize]);
            }
        }
    }

    // 使用 from_utf8_unchecked 提高性能，因为 BASE64_CHARS 都是有效的 ASCII 字符
    unsafe { String::from_utf8_unchecked(result) }
}

/// 将 Base64 字符串解码为字节数组。
///
/// # Arguments
///
/// * `input`: 要解码的 Base64 字符串
///
/// # Returns
///
/// 如果解码成功，返回 Some(解码后的字节数组)；如果输入无效，返回 None
pub fn from_base64(input: &str) -> Option<Vec<u8>> {
    let input = input.as_bytes();

    // 检查输入长度，Base64 编码的长度必须是 4 的倍数或余 2/3
    if input.is_empty() || input.len() % 4 == 1 {
        return None;
    }

    // 检查是否包含无效字符，无效字符直接返回None
    if input.iter().any(|&b| BASE64_LOOKUP[b as usize] == -1) {
        return None;
    }

    // 预分配足够容量，避免多次分配内存
    let capacity = input.len() / 4 * 3;
    let mut result = Vec::with_capacity(capacity);

    // 每四个字符为一组进行处理
    let mut chunks = input.chunks_exact(4);
    for chunk in &mut chunks {
        // 使用查找表将 Base64 字符转换为 6 位的值
        let n1 = BASE64_LOOKUP[chunk[0] as usize] as u32;
        let n2 = BASE64_LOOKUP[chunk[1] as usize] as u32;
        let n3 = BASE64_LOOKUP[chunk[2] as usize] as u32;
        let n4 = BASE64_LOOKUP[chunk[3] as usize] as u32;

        // 将四个 6 位的值合并为一个 u32，并拆分成三个字节
        let n = (n1 << 18) | (n2 << 12) | (n3 << 6) | n4;
        result.push((n >> 16) as u8);
        result.push(((n >> 8) & 0xFF) as u8);
        result.push((n & 0xFF) as u8);
    }

    // 处理剩余的字符
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let n1 = BASE64_LOOKUP[remainder[0] as usize] as u32;
        let n2 = BASE64_LOOKUP[remainder[1] as usize] as u32;

        let mut n = (n1 << 18) | (n2 << 12);
        result.push((n >> 16) as u8);

        // 如果剩余字符长度大于 2，则需要处理第二个字节
        if remainder.len() > 2 {
            let n3 = BASE64_LOOKUP[remainder[2] as usize] as u32;
            n |= n3 << 6;
            result.push(((n >> 8) & 0xFF) as u8);
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let test_cases = vec![
            vec![0u8, 1, 2, 3],
            vec![255u8, 254, 253],
            vec![0u8],
            vec![0u8, 1],
            vec![0u8, 1, 2],
            vec![255u8; 1000],
        ];

        for case in test_cases {
            let encoded = to_base64(&case);
            let decoded = from_base64(&encoded).unwrap();
            assert_eq!(case, decoded);
        }
    }

    #[test]
    fn test_invalid_input() {
        assert_eq!(from_base64(""), None); // 空字符串
        assert_eq!(from_base64("a"), None); // 长度为 1
        assert_eq!(from_base64("!@#$"), None); // 无效字符
        assert_eq!(from_base64("YWJj!"), None); // 包含无效字符
        assert!(from_base64("YWJj").is_some()); // 有效输入
    }
}
