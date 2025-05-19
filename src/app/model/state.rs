use crate::common::utils::{generate_checksum_with_repair, generate_hash};
use memmap2::{MmapMut, MmapOptions};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::OpenOptions,
};

use super::{
    super::lazy::{LOGS_FILE_PATH, TOKENS_FILE_PATH},
    LogStatus, RequestLog, TokenInfo,
    log::RequestLogHelper,
    proxy_pool::Proxies,
};

// 页面内容类型枚举
#[derive(Clone, Serialize, Deserialize, Archive, RkyvDeserialize, RkyvSerialize)]
#[serde(tag = "type", content = "content")]
pub enum PageContent {
    #[serde(rename = "default")]
    Default, // 默认行为
    #[serde(rename = "text")]
    Text(String), // 纯文本
    #[serde(rename = "html")]
    Html(String), // HTML 内容
}

impl Default for PageContent {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Default, Archive, RkyvDeserialize, RkyvSerialize)]
pub struct Pages {
    pub root_content: PageContent,
    pub logs_content: PageContent,
    pub config_content: PageContent,
    pub tokens_content: PageContent,
    pub proxies_content: PageContent,
    pub shared_styles_content: PageContent,
    pub shared_js_content: PageContent,
    pub about_content: PageContent,
    pub readme_content: PageContent,
    pub api_content: PageContent,
    pub build_key_content: PageContent,
}

// Token管理器
#[derive(Clone, Archive, RkyvDeserialize, RkyvSerialize)]
pub struct TokenManager {
    pub tokens: Vec<TokenInfo>,
    pub tags: HashSet<String>, // 存储所有已使用的标签
}

// 请求统计管理器
pub struct RequestStatsManager {
    pub total_requests: u64,
    pub active_requests: u64,
    pub error_requests: u64,
    pub request_logs: VecDeque<RequestLog>,
}

pub struct AppState {
    pub token_manager: TokenManager,
    pub request_manager: RequestStatsManager,
    pub proxies: Proxies,
}

impl TokenManager {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        let mut tags = HashSet::new();
        for token in &tokens {
            if let Some(token_tags) = &token.tags {
                tags.extend(token_tags.keys().cloned());
            }
        }

        Self { tokens, tags }
    }

    #[inline(always)]
    pub fn update_global_tags(&mut self, new_tags: &HashMap<String, Option<String>>) {
        // 将新标签添加到全局标签集合中
        self.tags.extend(new_tags.keys().cloned());
    }

    #[inline(always)]
    pub fn update_tokens_tags(
        &mut self,
        tokens: &[String],
        new_tags: Option<HashMap<String, Option<String>>>,
    ) -> Result<(), &'static str> {
        // 创建tokens的HashSet用于快速查找
        let tokens_set: HashSet<_> = tokens.iter().collect();

        // 更新指定tokens的标签
        for token_info in &mut self.tokens {
            if tokens_set.contains(&token_info.token) {
                token_info.tags = new_tags.clone();
            }
        }

        // 更新全局标签集合
        self.tags = self
            .tokens
            .iter()
            .filter_map(|t| t.tags.as_ref())
            .flat_map(|tags| tags.keys().cloned())
            .collect();

        Ok(())
    }

    #[inline(always)]
    pub fn get_tokens_by_tag(&self, tag: &str) -> Result<Vec<&TokenInfo>, &'static str> {
        if !self.tags.contains(tag) {
            return Err("Tag does not exist");
        }

        Ok(self
            .tokens
            .iter()
            .filter(|t| {
                t.tags
                    .as_ref()
                    .is_some_and(|tags| tags.keys().any(|t| t == tag))
            })
            .collect())
    }

    #[inline(always)]
    pub fn update_checksum(&mut self) {
        for token_info in self.tokens.iter_mut() {
            token_info.checksum = generate_checksum_with_repair(&token_info.checksum);
            token_info.client_key = Some(generate_hash());
        }
    }

    pub async fn save_tokens(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = rkyv::to_bytes::<_, 256>(self)?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&*TOKENS_FILE_PATH)?;

        if bytes.len() > usize::MAX / 2 {
            return Err("Token数据过大".into());
        }

        file.set_len(bytes.len() as u64)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap.copy_from_slice(&bytes);
        mmap.flush()?;

        Ok(())
    }

    pub async fn load_tokens() -> Result<Self, Box<dyn std::error::Error>> {
        let file = match OpenOptions::new().read(true).open(&*TOKENS_FILE_PATH) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::new(Vec::new()));
            }
            Err(e) => return Err(Box::new(e)),
        };

        if file.metadata()?.len() > usize::MAX as u64 {
            return Err("Token文件过大".into());
        }

        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let archived = unsafe { rkyv::archived_root::<Self>(&mmap) };
        Ok(archived.deserialize(&mut rkyv::Infallible)?)
    }
}

impl RequestStatsManager {
    pub fn new(request_logs: VecDeque<RequestLog>) -> Self {
        Self {
            total_requests: request_logs.len() as u64,
            active_requests: 0,
            error_requests: request_logs
                .iter()
                .filter(|log| matches!(log.status, LogStatus::Failure))
                .count() as u64,
            request_logs,
        }
    }

    pub async fn save_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = rkyv::to_bytes::<_, 256>(
            &self
                .request_logs
                .iter()
                .map(RequestLogHelper::from)
                .collect::<Vec<_>>(),
        )?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&*LOGS_FILE_PATH)?;

        if bytes.len() > usize::MAX / 2 {
            return Err("日志数据过大".into());
        }

        file.set_len(bytes.len() as u64)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap.copy_from_slice(&bytes);
        mmap.flush()?;

        Ok(())
    }

    pub async fn load_logs() -> Result<VecDeque<RequestLog>, Box<dyn std::error::Error>> {
        let file = match OpenOptions::new().read(true).open(&*LOGS_FILE_PATH) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(VecDeque::new());
            }
            Err(e) => return Err(Box::new(e)),
        };

        if file.metadata()?.len() > usize::MAX as u64 {
            return Err("日志文件过大".into());
        }

        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let archived = unsafe { rkyv::archived_root::<Vec<RequestLogHelper>>(&mmap) };
        let helper: Vec<RequestLogHelper> = archived.deserialize(&mut rkyv::Infallible)?;
        Ok(helper
            .into_iter()
            .map(RequestLogHelper::into_request_log)
            .collect())
    }
}

impl AppState {
    pub async fn new() -> Self {
        // 尝试加载保存的数据
        let logs = RequestStatsManager::load_logs().await.unwrap_or_default();
        let token_manager = TokenManager::load_tokens()
            .await
            .unwrap_or(TokenManager::new(Vec::new()));
        let mut proxies = Proxies::load_proxies().await.unwrap_or(Proxies::new());

        // 更新全局代理池
        if let Err(e) = proxies.update_global_pool() {
            eprintln!("更新全局代理池失败: {e}");
        }

        Self {
            token_manager,
            request_manager: RequestStatsManager::new(logs),
            proxies,
        }
    }

    pub async fn save_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 并行保存 logs、tokens 和 proxies
        let (logs_result, tokens_result, proxies_result) = tokio::join!(
            self.request_manager.save_logs(),
            self.token_manager.save_tokens(),
            self.proxies.save_proxies()
        );

        logs_result?;
        tokens_result?;
        proxies_result?;
        Ok(())
    }
}
