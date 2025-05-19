use crate::core::model::Role;

#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
enum ErrorInfoHelper {
    None,
    Error(String),
    Details { error: String, details: String },
}
impl From<ErrorInfoHelper> for super::ErrorInfo {
    #[inline]
    fn from(helper: ErrorInfoHelper) -> Self {
        match helper {
            ErrorInfoHelper::None => Self::None,
            ErrorInfoHelper::Error(e) => Self::new(&e),
            ErrorInfoHelper::Details { error, details } => Self::new_details(&error, &details),
        }
    }
}
impl From<super::ErrorInfo> for ErrorInfoHelper {
    #[inline]
    fn from(ori: super::ErrorInfo) -> Self {
        match ori {
            super::ErrorInfo::None => Self::None,
            super::ErrorInfo::Error(e) => Self::Error(e.to_string()),
            super::ErrorInfo::Details { error, details } => Self::Details {
                error: error.to_string(),
                details: details.to_string(),
            },
        }
    }
}
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub(super) struct RequestLogHelper {
    id: u64,
    timestamp: chrono::DateTime<chrono::Local>,
    model: String,
    token_info: super::TokenInfo,
    chain: Option<ChainHelper>,
    timing: super::TimingInfo,
    stream: bool,
    status: super::LogStatus,
    error: ErrorInfoHelper,
}
impl RequestLogHelper {
    #[inline]
    pub(super) fn into_request_log(self) -> super::RequestLog {
        super::RequestLog {
            id: self.id,
            timestamp: self.timestamp,
            model: crate::leak::intern_string(self.model),
            token_info: self.token_info,
            chain: self.chain.map(Into::into),
            timing: self.timing,
            stream: self.stream,
            status: self.status,
            error: self.error.into(),
        }
    }
}
impl From<&super::RequestLog> for RequestLogHelper {
    #[inline]
    fn from(log: &super::RequestLog) -> Self {
        Self {
            id: log.id,
            timestamp: log.timestamp,
            model: log.model.to_string(),
            token_info: log.token_info.clone(),
            chain: log.chain.clone().map(Into::into),
            timing: log.timing,
            stream: log.stream,
            status: log.status,
            error: log.error.into(),
        }
    }
}
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct PromptMessageHelper {
    role: Role,
    content: String,
}
impl From<PromptMessageHelper> for super::PromptMessage {
    #[inline]
    fn from(helper: PromptMessageHelper) -> Self {
        match helper.role {
            Role::System => super::PromptMessage {
                role: helper.role,
                content: super::PromptContent::Leaked(crate::leak::intern_string(helper.content)),
            },
            _ => super::PromptMessage {
                role: helper.role,
                content: super::PromptContent::Shared(super::RODEO.get_or_intern(helper.content)),
            },
        }
    }
}
impl From<super::PromptMessage> for PromptMessageHelper {
    #[inline]
    fn from(ori: super::PromptMessage) -> Self {
        Self {
            role: ori.role,
            content: ori.content.into_owned(),
        }
    }
}
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum PromptHelper {
    None,
    Origin(String),
    Parsed(Vec<PromptMessageHelper>),
}
impl From<PromptHelper> for super::Prompt {
    #[inline]
    fn from(helper: PromptHelper) -> Self {
        match helper {
            PromptHelper::None => Self::None,
            PromptHelper::Origin(s) => Self::Origin(s),
            PromptHelper::Parsed(v) => {
                Self::Parsed(v.into_iter().map(Into::into).collect::<Vec<_>>())
            }
        }
    }
}
impl From<super::Prompt> for PromptHelper {
    #[inline]
    fn from(ori: super::Prompt) -> Self {
        match ori {
            super::Prompt::None => Self::None,
            super::Prompt::Origin(s) => Self::Origin(s),
            super::Prompt::Parsed(v) => {
                Self::Parsed(v.into_iter().map(Into::into).collect::<Vec<_>>())
            }
        }
    }
}
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct ChainHelper {
    pub prompt: PromptHelper,
    pub delays: Option<(String, Vec<(u32, f32)>)>,
    pub usage: super::OptionUsage,
}
impl From<ChainHelper> for super::Chain {
    #[inline]
    fn from(helper: ChainHelper) -> Self {
        Self {
            prompt: helper.prompt.into(),
            delays: helper.delays,
            usage: helper.usage,
        }
    }
}
impl From<super::Chain> for ChainHelper {
    #[inline]
    fn from(ori: super::Chain) -> Self {
        Self {
            prompt: ori.prompt.into(),
            delays: ori.delays,
            usage: ori.usage,
        }
    }
}
