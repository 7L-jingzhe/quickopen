/// 展开环境变量（如 %APPDATA%, $HOME）
pub fn expand_path(path: &str) -> String {
    shellexpand::full(path)
        .unwrap_or_else(|_| path.into())
        .to_string()
}

/// 判断是否为 URL
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("file://")
}
