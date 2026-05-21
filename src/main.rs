use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 配置文件结构
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    aliases: HashMap<String, String>,
}

impl Config {
    /// 从文件加载配置，如果文件不存在则返回空配置（不自动创建）
    fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse TOML from {:?}", path))?;
        Ok(config)
    }

    /// 保存配置到文件
    fn save_to_file(&self, path: &Path) -> Result<()> {
        let contents =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;
        fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        Ok(())
    }
}

/// 获取配置文件路径：优先当前目录下的 quickopen.toml，其次 config.toml，最后用户主目录下的 .quickopen.toml
fn get_config_path() -> Result<PathBuf> {
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
fn ensure_config_exists(path: &Path) -> Result<()> {
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

/// 展开环境变量（如 %APPDATA%, $HOME）
fn expand_path(path: &str) -> String {
    shellexpand::full(path)
        .unwrap_or_else(|_| path.into())
        .to_string()
}

/// 判断是否为 URL
fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("file://")
}

/// 用资源管理器打开目录
fn open_directory_with_explorer(path: &Path) -> Result<()> {
    let abs_path = path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", path))?;
    let path_str = abs_path.to_str().context("Invalid directory path")?;
    Command::new("explorer")
        .arg(path_str)
        .spawn()
        .context("Failed to open directory")?;
    Ok(())
}

/// 运行可执行文件（支持传递参数），后台运行，不阻塞
fn run_executable(path: &Path, args: &[String]) -> Result<()> {
    let path_str = path.to_str().context("Invalid executable path")?;
    let mut cmd = Command::new(path_str);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let child = cmd
        .spawn()
        .with_context(|| format!("Failed to run: {}", path_str))?;
    drop(child);
    Ok(())
}

/// 用默认浏览器打开 URL（通过 Windows start 命令）
fn open_url_with_start(url: &str) -> Result<()> {
    let status = Command::new("cmd")
        .args(&["/C", "start", "", url])
        .status()
        .context("Failed to execute start command")?;
    if !status.success() {
        bail!("Failed to open URL with start");
    }
    Ok(())
}

/// 拆分目标字符串，返回 (可执行文件路径, 默认参数列表)
/// 支持路径中包含空格（使用双引号包裹）
/// 示例：
/// - "D:/Tool/app.exe --arg1 val"  => ("D:/Tool/app.exe", ["--arg1", "val"])
/// - "\"D:/Program Files/app.exe\" --arg1 val" => ("D:/Program Files/app.exe", ["--arg1", "val"])
fn split_target(target: &str) -> (String, Vec<String>) {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return (String::new(), vec![]);
    }

    // 如果以双引号开头，找闭合的双引号
    if trimmed.starts_with('"') {
        let after_first_quote = &trimmed[1..];
        if let Some(close_pos) = after_first_quote.find('"') {
            let path = after_first_quote[..close_pos].to_string();
            let rest = after_first_quote[close_pos + 1..].trim();
            let args = if rest.is_empty() {
                vec![]
            } else {
                rest.split_whitespace().map(|s| s.to_string()).collect()
            };
            return (path, args);
        }
    }

    // 否则按第一个空格分割路径和参数
    if let Some(space_pos) = trimmed.find(char::is_whitespace) {
        let path = trimmed[..space_pos].to_string();
        let rest = trimmed[space_pos..].trim();
        let args = if rest.is_empty() {
            vec![]
        } else {
            rest.split_whitespace().map(|s| s.to_string()).collect()
        };
        (path, args)
    } else {
        (trimmed.to_string(), vec![])
    }
}

/// 执行打开操作（支持目标内嵌参数）
fn execute_open(alias: &str, extra_args: &[String], config: &Config) -> Result<()> {
    let target = match config.aliases.get(alias) {
        Some(t) => t,
        None => {
            bail!("Alias '{}' not found in config", alias);
        }
    };
    let expanded = expand_path(target);

    // 处理 URL
    if is_url(&expanded) {
        return open_url_with_start(&expanded);
    }

    // 拆分目标路径和默认参数
    let (exe_path_str, default_args) = split_target(&expanded);
    if exe_path_str.is_empty() {
        bail!("Invalid target for alias '{}'", alias);
    }
    let path = Path::new(&exe_path_str);

    // 目录处理
    if path.is_dir() {
        return open_directory_with_explorer(path);
    }

    // 文件存在性检查
    if !path.exists() {
        bail!("Target does not exist: {}", exe_path_str);
    }

    // 合并默认参数和用户额外参数（默认参数在前，用户额外参数在后）
    let mut all_args = default_args;
    all_args.extend(extra_args.iter().cloned());

    run_executable(path, &all_args)
}

/// 列出所有别名
fn list_aliases(config: &Config) {
    if config.aliases.is_empty() {
        println!("No aliases found.");
    } else {
        println!("Aliases:");
        for (alias, target) in &config.aliases {
            println!("  {} -> {}", alias, target);
        }
    }
}

/// 添加别名（如果已存在则询问是否覆盖）
fn add_alias(config: &mut Config, alias: &str, target: &str, config_path: &Path) -> Result<()> {
    if config.aliases.contains_key(alias) {
        print!(
            "Alias '{}' already exists (target: {}). Overwrite? [y/N]: ",
            alias, config.aliases[alias]
        );
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Canceled.");
            return Ok(());
        }
    }
    config.aliases.insert(alias.to_string(), target.to_string());
    config.save_to_file(config_path)?;
    println!("Alias '{}' added/updated.", alias);
    Ok(())
}

/// 移除别名
fn remove_alias(config: &mut Config, alias: &str, config_path: &Path) -> Result<()> {
    if config.aliases.remove(alias).is_some() {
        config.save_to_file(config_path)?;
        println!("Alias '{}' removed.", alias);
    } else {
        println!("Alias '{}' not found.", alias);
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  quickopen <alias> [args...]         - Open the alias");
        eprintln!("  quickopen list                      - List all aliases");
        eprintln!("  quickopen add <alias> <target>      - Add or update an alias");
        eprintln!("  quickopen remove <alias>            - Remove an alias");
        eprintln!();
        eprintln!("Config file search order: ./quickopen.toml, ./config.toml, ~/.quickopen.toml");
        std::process::exit(1);
    }

    let config_path = get_config_path()?;
    ensure_config_exists(&config_path)?;

    let mut config = Config::load_from_file(&config_path)?;

    match args[1].as_str() {
        "list" => {
            list_aliases(&config);
        }
        "add" => {
            if args.len() < 4 {
                eprintln!("Usage: quickopen add <alias> <target>");
                std::process::exit(1);
            }
            let alias = &args[2];
            let target = &args[3];
            add_alias(&mut config, alias, target, &config_path)?;
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: quickopen remove <alias>");
                std::process::exit(1);
            }
            let alias = &args[2];
            remove_alias(&mut config, alias, &config_path)?;
        }
        _ => {
            // 普通打开操作
            let alias = &args[1];
            let extra_args = &args[2..];
            execute_open(alias, extra_args, &config)?;
        }
    }

    Ok(())
}
