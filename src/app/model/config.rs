use memmap2::{MmapMut, MmapOptions};
use rkyv::{archived_root, Deserialize as _};
use std::fs::OpenOptions;

use crate::app::lazy::{LOGS_FILE_PATH, PAGES_FILE_PATH};

use super::{AppConfig, AppState, Pages, RequestLog, APP_CONFIG};

impl AppState {
    // 保存日志的方法
    pub(crate) async fn save_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 序列化日志
        let bytes = rkyv::to_bytes::<_, 256>(&self.request_logs)?;

        // 创建或打开文件
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(LOGS_FILE_PATH.as_str())?;

        // 添加大小检查
        if bytes.len() > usize::MAX / 2 {
            return Err("日志数据过大".into());
        }

        // 设置文件大小
        file.set_len(bytes.len() as u64)?;

        // 创建可写入的内存映射
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        // 写入数据
        mmap.copy_from_slice(&bytes);

        // 同步到磁盘
        mmap.flush()?;

        Ok(())
    }

    // 加载日志的方法
    pub(super) async fn load_saved_logs() -> Result<Vec<RequestLog>, Box<dyn std::error::Error>> {
        let file = match OpenOptions::new().read(true).open(LOGS_FILE_PATH.as_str()) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(Box::new(e)),
        };

        // 添加文件大小检查
        if file.metadata()?.len() > usize::MAX as u64 {
            return Err("日志文件过大".into());
        }

        // 创建只读内存映射
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        // 验证并反序列化数据
        let archived = unsafe { archived_root::<Vec<RequestLog>>(&mmap) };
        Ok(archived.deserialize(&mut rkyv::Infallible)?)
    }
}

impl AppConfig {
    pub fn save_config() -> Result<(), Box<dyn std::error::Error>> {
        let pages = APP_CONFIG.read().pages.clone();
        let bytes = rkyv::to_bytes::<_, 256>(&pages)?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(PAGES_FILE_PATH.as_str())?;

        // 添加大小检查
        if bytes.len() > usize::MAX / 2 {
            return Err("配置数据过大".into());
        }

        file.set_len(bytes.len() as u64)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap.copy_from_slice(&bytes);
        mmap.flush()?;

        Ok(())
    }

    pub fn load_saved_config() -> Result<(), Box<dyn std::error::Error>> {
        let file = match OpenOptions::new().read(true).open(PAGES_FILE_PATH.as_str()) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(e) => return Err(Box::new(e)),
        };

        // 添加文件大小检查
        if file.metadata()?.len() > usize::MAX as u64 {
            return Err("配置文件过大".into());
        }

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        let archived = unsafe { archived_root::<Pages>(&mmap) };
        let pages = archived.deserialize(&mut rkyv::Infallible)?;
        let mut config = APP_CONFIG.write();
        config.pages = pages;

        Ok(())
    }
}
