//! CLI 参数定义（clap derive）。

use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "prompt-cli",
    version,
    about = "提示词管理平台 CLI 输出工具（开发版）",
    long_about = "从本地 prompt.db 读取提示词，支持 list / get / export / run，以及 help 帮助子命令。",
    disable_help_subcommand = false
)]
pub struct Cli {
    /// 数据库文件路径（缺省当前目录 prompt.db）
    #[arg(long, global = true, default_value = "prompt.db")]
    pub db: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 列出提示词（可按分类 / 标签 / 项目 / 关键词过滤）
    List {
        /// 按分类（名或 id）过滤
        #[arg(long)]
        category: Option<String>,
        /// 按标签过滤（可多次，命中需同时包含）
        #[arg(long)]
        tag: Vec<String>,
        /// 按项目过滤
        #[arg(long)]
        project: Option<String>,
        /// 关键词
        #[arg(long)]
        keyword: Option<String>,
        /// 输出格式
        #[arg(long, value_parser = ["table", "json"], default_value = "table")]
        format: String,
    },
    /// 读取单条提示词
    Get {
        id: String,
        /// 输出格式（skill 输出 SKILL.md 文本）
        #[arg(long, value_parser = ["skill", "markdown", "json"], default_value = "markdown")]
        format: String,
    },
    /// 按分类 / 标签 / 项目批量导出
    Export {
        /// 按分类导出
        #[arg(long)]
        category: Option<String>,
        /// 按标签导出（可多次）
        #[arg(long)]
        tag: Vec<String>,
        /// 按项目导出
        #[arg(long)]
        project: Option<String>,
        /// 导出格式
        #[arg(long, value_parser = ["skill", "markdown", "json"], default_value = "skill")]
        format: String,
        /// 输出目录
        #[arg(long, default_value = "./prompt-export")]
        out: PathBuf,
        /// 使用变量默认值渲染
        #[arg(long)]
        render: bool,
        /// 变量填值（name=value，可多次）
        #[arg(long)]
        var: Vec<String>,
    },
    /// 渲染单条模板提示词为成品
    Run {
        id: String,
        /// 变量填值（name=value，可多次）
        #[arg(long)]
        var: Vec<String>,
        /// 输出文件（缺省打印到 stdout）
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

/// 解析 `name=value` 列表为变量映射。
pub fn parse_vars(pairs: &[String]) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for p in pairs {
        let (k, v) = p
            .split_once('=')
            .ok_or_else(|| format!("变量格式应为 name=value：{p}"))?;
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}
