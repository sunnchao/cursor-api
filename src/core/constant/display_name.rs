/// 计算 AI 模型标识符的显示名称。
///
/// 规则：
/// 1. 单个数字通过 '-' 连接时 (前后都不是数字)，'-' 变为 '.' (例如 "3-5" -> "3.5")。
/// 2. 日期格式中的 '-' (如 YYYY-MM-DD, MM-DD) 保持不变。
/// 3. 其他所有的 '-' 都被替换为空格 ' '。
/// 4. 由原始 '-' 分隔的各部分（处理后）首字母大写 (Title Case)。
/// 5. 特殊规则：如果原始标识符以 "gpt" 开头，则输出的对应部分为 "GPT"。
///
/// # Arguments
///
/// * `identifier` - AI 模型的原始标识符字符串。
///
/// # Returns
///
/// * `String` - 计算得到的显示名称。
///
/// # Examples
///
/// ```
/// assert_eq!(calculate_display_name_v3("claude-3-5-sonnet"), "Claude 3.5 Sonnet");
/// assert_eq!(calculate_display_name_v3("gpt-4-turbo-2024-04-09"), "GPT 4 Turbo 2024-04-09"); // 日期 '-' 不变
/// assert_eq!(calculate_display_name_v3("gemini-1.5-flash-500k"), "Gemini 1.5 Flash 500k");
/// assert_eq!(calculate_display_name_v3("deepseek-v3"), "Deepseek V3");
/// assert_eq!(calculate_display_name_v3("gpt-4o"), "GPT 4o");
/// assert_eq!(calculate_display_name_v3("gpt-3.5-turbo"), "GPT 3.5 Turbo"); // 输入有 .
/// assert_eq!(calculate_display_name_v3("gemini-2.5-pro-exp-03-25"), "Gemini 2.5 Pro Exp 03-25"); // 日期 '-' 不变
/// assert_eq!(calculate_display_name_v3("version-10-beta"), "Version 10 Beta"); // 10 不是单数字
/// assert_eq!(calculate_display_name_v3("model-1-test-9-case"), "Model 1 Test 9 Case"); // d-d 转义被空格隔开
/// ```
pub fn calculate_display_name_v3(identifier: &str) -> String {
    if identifier.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut capitalize_next = true;
    let mut prev_char: Option<char> = None;
    let mut prev_prev_char: Option<char> = None;

    let mut char_iter = identifier.chars().peekable();
    if identifier.starts_with("gpt-") {
        result.push_str("GPT");
        for _ in 0..4 {
            char_iter.next();
        }
        result.push(' ');
        capitalize_next = true;
        prev_char = Some('-');
        prev_prev_char = Some('t');
    } else if identifier == "gpt" {
        return "GPT".to_string();
    }

    while let Some(current_char) = char_iter.next() {
        match current_char {
            '-' => {
                let prev_is_digit = prev_char.map_or(false, |p| p.is_ascii_digit());
                let next_is_digit = char_iter.peek().map_or(false, |&n| n.is_ascii_digit());

                if prev_is_digit && next_is_digit {
                    let prev_prev_is_digit = prev_prev_char.map_or(false, |pp| pp.is_ascii_digit());

                    let mut next_next_is_digit = false;
                    let mut lookahead = char_iter.clone();
                    lookahead.next();
                    if let Some(next_next_char) = lookahead.peek() {
                        if next_next_char.is_ascii_digit() {
                            next_next_is_digit = true;
                        }
                    }

                    if prev_prev_is_digit || next_next_is_digit {
                        result.push('-');
                        capitalize_next = true;
                    } else {
                        result.push('.');
                        capitalize_next = false;
                    }
                } else {
                    result.push(' ');
                    capitalize_next = true;
                }
                prev_prev_char = prev_char;
                prev_char = Some('-');
            }
            c => {
                if capitalize_next {
                    result.extend(c.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
                prev_prev_char = prev_char;
                prev_char = Some(c);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_v3_final() {
        // Anthropic
        assert_eq!(calculate_display_name_v3("claude-3-opus"), "Claude 3 Opus");
        assert_eq!(
            calculate_display_name_v3("claude-3.5-sonnet"),
            "Claude 3.5 Sonnet"
        ); // Input has dot
        assert_eq!(
            calculate_display_name_v3("claude-3-5-sonnet"),
            "Claude 3.5 Sonnet"
        ); // d-d conversion
        assert_eq!(
            calculate_display_name_v3("claude-3-haiku-200k"),
            "Claude 3 Haiku 200k"
        );

        // OpenAI (GPT & Date Preservation)
        assert_eq!(calculate_display_name_v3("gpt-4"), "GPT 4");
        assert_eq!(calculate_display_name_v3("gpt-4o"), "GPT 4o");
        assert_eq!(calculate_display_name_v3("gpt-3.5-turbo"), "GPT 3.5 Turbo"); // Input has dot
        assert_eq!(
            calculate_display_name_v3("gpt-4-turbo-2024-04-09"),
            "GPT 4 Turbo 2024-04-09"
        ); // Date preserved!
        assert_eq!(calculate_display_name_v3("gpt-4o-mini"), "GPT 4o Mini");
        assert_eq!(calculate_display_name_v3("gpt"), "GPT");
        assert_eq!(calculate_display_name_v3("gpt-"), "GPT "); // Trailing hyphen becomes space

        // Google (Date Preservation)
        assert_eq!(
            calculate_display_name_v3("gemini-1.5-flash-500k"),
            "Gemini 1.5 Flash 500k"
        ); // Input has dot
        assert_eq!(
            calculate_display_name_v3("gemini-2.5-pro-exp-03-25"),
            "Gemini 2.5 Pro Exp 03-25"
        ); // Date preserved!

        // Deepseek
        assert_eq!(calculate_display_name_v3("deepseek-v3"), "Deepseek V3");

        // Other & Edge Cases
        assert_eq!(calculate_display_name_v3("o1-mini"), "O1 Mini");
        assert_eq!(
            calculate_display_name_v3("model-1-test-9-case"),
            "Model 1 Test 9 Case"
        ); // d-d handled
        assert_eq!(
            calculate_display_name_v3("version-10-beta"),
            "Version 10 Beta"
        ); // 10 is not single digit
        assert_eq!(
            calculate_display_name_v3("alpha-1-5-omega"),
            "Alpha 1.5 Omega"
        ); // d-d handled
        assert_eq!(calculate_display_name_v3("my-gpt-model"), "My Gpt Model"); // gpt not at start
        assert_eq!(calculate_display_name_v3(""), ""); // Empty
        assert_eq!(calculate_display_name_v3("-start-"), " Start "); // Leading/trailing hyphens
        assert_eq!(calculate_display_name_v3("a-b-c"), "A B C");
        assert_eq!(calculate_display_name_v3("3-5"), "3.5"); // Only d-d
        assert_eq!(calculate_display_name_v3("2024-release"), "2024 Release"); // Number-text
        assert_eq!(calculate_display_name_v3("data-01-01"), "Data 01-01"); // Date like MM-DD
    }
}
