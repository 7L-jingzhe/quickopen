mod alias;
mod config;
mod launcher;
mod utils;

use alias::{add_alias, list_aliases, remove_alias};
use anyhow::Result;
use config::{ensure_config_exists, get_config_path, Config};
use launcher::{open_directory_with_explorer, open_url_with_start, run_executable, split_target};
use std::env;
use utils::{expand_path, is_url};

/// 执行打开操作（支持目标内嵌参数）
fn execute_open(alias: &str, extra_args: &[String], config: &Config) -> Result<()> {
    let target = match config.aliases.get(alias) {
        Some(t) => t,
        None => {
            anyhow::bail!("Alias '{}' not found in config", alias);
        }
    };
    let expanded = expand_path(target);

    if is_url(&expanded) {
        return open_url_with_start(&expanded);
    }

    let (exe_path_str, default_args) = split_target(&expanded);
    if exe_path_str.is_empty() {
        anyhow::bail!("Invalid target for alias '{}'", alias);
    }
    let path = std::path::Path::new(&exe_path_str);

    if path.is_dir() {
        return open_directory_with_explorer(path);
    }

    if !path.exists() {
        anyhow::bail!("Target does not exist: {}", exe_path_str);
    }

    let mut all_args = default_args;
    all_args.extend(extra_args.iter().cloned());
    run_executable(path, &all_args)
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
            let alias = &args[1];
            let extra_args = &args[2..];
            execute_open(alias, extra_args, &config)?;
        }
    }

    Ok(())
}
