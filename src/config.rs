use anyhow::{Context, Result};
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// 配置文件结构
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub aliases: HashMap<String, String>,
}

impl Config {
    /// 从文件加载配置
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse TOML from {:?}", path))?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let contents =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;
        fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        Ok(())
    }
}

/// 获取配置文件路径：优先当前目录下的 quickopen.toml，其次 config.toml，最后用户主目录下的 .quickopen.toml
pub fn get_config_path() -> Result<PathBuf> {
    let cwd = env::current_dir().context("Failed to get current directory")?;
    // 1. 当前目录下的 quickopen.toml
    let local = cwd.join("quickopen.toml");
    if local.exists() {
        return Ok(local);
    }
    // 2. 当前目录下的 config.toml
    let local_alt = cwd.join("config.toml");
    if local_alt.exists() {
        return Ok(local_alt);
    }
    // 3. 用户主目录下的 .quickopen.toml
    let home = dirs::home_dir().context("Cannot find home directory")?;
    Ok(home.join(".quickopen.toml"))
}

/// 确保配置文件存在（如果不存在则创建默认配置）
pub fn ensure_config_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_default_config(path)?;
        eprintln!("Created default config file at: {:?}", path);
        eprintln!("Please edit it to add your own aliases, then run the program again.");
        std::process::exit(0);
    }
    Ok(())
}

/// 创建默认配置文件（在传入的路径位置创建）
fn create_default_config(path: &Path) -> Result<()> {
    let default_content = r#"# QuickOpen 配置文件
# 格式: 别名 = "目标路径或URL"
# 支持环境变量: %APPDATA%, %USERPROFILE%, $HOME 等
# 路径可以使用正斜杠 / 或双反斜杠 \\
# 如果目标需要参数，可以直接写在路径后面（如 "D:/app.exe --arg1 val"）
# 如果路径包含空格，请用双引号包裹路径部分，例如 "\"D:/Program Files/app.exe\" --arg"

[aliases]
# 示例：游戏启动器
ys = "D:/Games/miHoYo/miHoYo Launcher/launcher.exe"

# 示例：带参数的程序
vm = "\"D:/Tool/VMware/vmware.exe\" --locale zh_CN"

# 示例：常用程序（系统 PATH 中的程序可以直接写名字）
note = "notepad.exe"

# 示例：文件夹
downloads = "D:/downloads"

# 示例：URL
github = "https://github.com"
rust_book = "https://doc.rust-lang.org/book/"
"#;
    fs::write(path, default_content)
        .with_context(|| format!("Failed to create default config file: {:?}", path))?;
    Ok(())
}
