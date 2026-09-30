use crate::errors::AppError;

/// 远程存储抽象（WebDAV 客户端的最小封装，M4 分工清单 §6 轨道 C）。
///
/// `SyncEngine::sync_once` 依赖它做远端读写，不感知 WebDAV 协议细节。
/// 方法名对齐 WebDAV 动词：put / get / propfind / mkcol。
pub trait RemoteStore: Send + Sync {
    /// 上传一个文件（PUT）。
    fn put(&self, path: &str, data: &[u8]) -> Result<(), AppError>;
    /// 下载一个文件（GET）。
    fn get(&self, path: &str) -> Result<Vec<u8>, AppError>;
    /// 列出目录下条目（PROPFIND），返回相对路径列表。
    fn propfind(&self, path: &str) -> Result<Vec<String>, AppError>;
    /// 创建目录（MKCOL）。
    fn mkcol(&self, path: &str) -> Result<(), AppError>;
}
