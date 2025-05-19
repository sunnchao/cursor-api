// include!(concat!(env!("OUT_DIR"), "/aiserver.v1.rs"));
include!("v1/aiserver.v1.rs");
use error_details::Error;

impl ErrorDetails {
    pub fn status_code(&self) -> u16 {
        match Error::try_from(self.error) {
            Ok(error) => match error {
                // 认证/授权相关错误
                Error::BadApiKey
                | Error::BadUserApiKey
                | Error::InvalidAuthId
                | Error::AuthTokenNotFound
                | Error::AuthTokenExpired
                | Error::Unauthorized => 401,

                // 权限不足
                Error::NotLoggedIn
                | Error::NotHighEnoughPermissions
                | Error::AgentRequiresLogin
                | Error::ProUserOnly
                | Error::TaskNoPermissions => 403,

                // 资源未找到
                Error::NotFound
                | Error::UserNotFound
                | Error::TaskUuidNotFound
                | Error::AgentEngineNotFound
                | Error::GitgraphNotFound
                | Error::FileNotFound => 404,

                // 请求过多/速率限制
                Error::FreeUserRateLimitExceeded
                | Error::ProUserRateLimitExceeded
                | Error::OpenaiRateLimitExceeded
                | Error::OpenaiAccountLimitExceeded
                | Error::GenericRateLimitExceeded
                | Error::Gpt4VisionPreviewRateLimit
                | Error::ApiKeyRateLimit => 429,

                // 客户端请求错误
                Error::BadRequest
                | Error::BadModelName
                | Error::SlashEditFileTooLong
                | Error::FileUnsupported
                | Error::ClaudeImageTooLarge
                | Error::ConversationTooLong => 400,

                // 超时
                Error::Timeout => 504,

                // 版本/弃用相关
                Error::Deprecated | Error::OutdatedClient => 410,

                // 资源耗尽/配额限制
                Error::FreeUserUsageLimit
                | Error::ProUserUsageLimit
                | Error::ResourceExhausted
                | Error::MaxTokens => 503,

                // OpenAI相关错误
                Error::Openai | Error::ApiKeyNotSupported => 500,

                // 客户端主动取消
                Error::UserAbortedRequest => 500,

                // 自定义消息
                Error::CustomMessage => 500,

                // 价格相关
                Error::UsagePricingRequired | Error::UsagePricingRequiredChangeable => 402,

                // 代码/仓库相关
                Error::RepositoryServiceRepositoryIsNotInitialized => 500,

                // 其他/未分类的服务器内部错误
                Error::Unspecified | Error::Debounced => 500,
            },
            Err(_) => 500,
        }
    }

    // pub fn is_expected(&self) -> bool {
    //     self.is_expected.unwrap_or_default()
    // }
}
