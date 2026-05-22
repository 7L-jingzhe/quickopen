use crate::config::Config;
use anyhow::Result;
use std::io::{self, Write};
use std::path::Path;

/// 列出所有别名
pub fn list_aliases(config: &Config) {
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
pub fn add_alias(config: &mut Config, alias: &str, target: &str, config_path: &Path) -> Result<()> {
    // 将 Windows 反斜杠转换为正斜杠
    let target = target.replace('\\', "/");

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
pub fn remove_alias(config: &mut Config, alias: &str, config_path: &Path) -> Result<()> {
    if config.aliases.remove(alias).is_some() {
        config.save_to_file(config_path)?;
        println!("Alias '{}' removed.", alias);
    } else {
        println!("Alias '{}' not found.", alias);
    }
    Ok(())
}
