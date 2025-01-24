pub struct DefaultModel {
    pub default_on: bool,
    pub is_long_context_only: Option<bool>,
    pub name: &'static str,
}

pub const AVAILABLE_MODELS2: [DefaultModel; 22] = [
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(false),
        name: CLAUDE_3_5_SONNET,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: GPT_4,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: None,
        name: GPT_4O,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: CLAUDE_3_OPUS,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: CURSOR_FAST,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: CURSOR_SMALL,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: GPT_3_5_TURBO,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: GPT_4_TURBO_2024_04_09,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(true),
        name: GPT_4O_128K,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(true),
        name: GEMINI_1_5_FLASH_500K,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(true),
        name: CLAUDE_3_HAIKU_200K,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(true),
        name: CLAUDE_3_5_SONNET_200K,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: Some(false),
        name: CLAUDE_3_5_SONNET_20241022,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(false),
        name: GPT_4O_MINI,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(false),
        name: O1_MINI,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(false),
        name: O1_PREVIEW,
    },
    DefaultModel {
        default_on: true,
        is_long_context_only: Some(false),
        name: O1,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: Some(false),
        name: CLAUDE_3_5_HAIKU,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: GEMINI_EXP_1206,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: GEMINI_2_0_FLASH_THINKING_EXP,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: GEMINI_2_0_FLASH_EXP,
    },
    DefaultModel {
        default_on: false,
        is_long_context_only: None,
        name: DEEPSEEK_V3,
    },
];