// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;

/// 把启动期的 panic / 错误落地到临时目录，便于在“双击快捷方式闪退”时定位真实原因。
fn log_line(msg: &str) {
    let path = std::env::temp_dir().join("prompt-manager-error.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{}", msg);
    }
    eprintln!("{}", msg);
}

fn main() {
    // 捕获所有 panic（含 IPC / 事件循环中的 panic），写进日志便于定位。
    std::panic::set_hook(Box::new(|info| {
        let mut line = format!("PANIC: {}", info);
        if let Some(loc) = info.location() {
            line.push_str(&format!(
                " @ {}:{}:{}",
                loc.file(),
                loc.line(),
                loc.column()
            ));
        }
        log_line(&line);
    }));

    match prompt_registry_lib::run() {
        Ok(()) => {
            // 正常退出：应用窗口被关闭（事件循环正常结束）。
            log_line("RUN_OK: app event loop exited normally");
        }
        Err(e) => {
            log_line(&format!("RUN_ERROR: {e:#}"));
            std::process::exit(1);
        }
    }
}
