use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// 用资源管理器打开目录
pub fn open_directory_with_explorer(path: &Path) -> Result<()> {
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
/// 如果遇到权限不足（错误 740），会自动以管理员身份重试
pub fn run_executable(path: &Path, args: &[String]) -> Result<()> {
    let path_str = path.to_str().context("Invalid executable path")?;
    let mut cmd = Command::new(path_str);
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    match cmd.spawn() {
        Ok(child) => {
            drop(child);
            Ok(())
        }
        Err(e) => {
            if let Some(code) = e.raw_os_error() {
                if code == 740 {
                    eprintln!(
                        "Requires administrator privileges. Attempting to run as administrator..."
                    );
                    // 构建参数列表部分（仅当非空时添加 -ArgumentList）
                    let arg_list_part = if args.is_empty() {
                        String::new()
                    } else {
                        let args_str = args
                            .iter()
                            .map(|s| format!("'{}'", s.replace('\'', "''")))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(" -ArgumentList @({})", args_str)
                    };
                    let ps_command = format!(
                        "Start-Process -FilePath '{}' -Verb RunAs{}",
                        path_str.replace('\'', "''"),
                        arg_list_part
                    );
                    let status = Command::new("powershell")
                        .args(&["-Command", &ps_command])
                        .status()
                        .context("Failed to elevate privileges")?;
                    if status.success() {
                        Ok(())
                    } else {
                        bail!("Failed to run with administrator privileges.");
                    }
                } else {
                    Err(e).with_context(|| format!("Failed to run: {}", path_str))
                }
            } else {
                Err(e).with_context(|| format!("Failed to run: {}", path_str))
            }
        }
    }
}

/// 用默认浏览器打开 URL（通过 Windows start 命令）
pub fn open_url_with_start(url: &str) -> Result<()> {
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
pub fn split_target(target: &str) -> (String, Vec<String>) {
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
