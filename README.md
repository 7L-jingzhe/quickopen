# 命令行快速启动工具

# QuickOpen

命令行快速启动工具，通过短别名打开程序、文件夹或网址。

## 功能

- 用别名代替长路径，快速启动
- 支持打开 `.exe`、文件夹（资源管理器）、网址（默认浏览器）
- 可向程序传递额外参数
- 命令行管理别名：`list`、`add`、`remove`


## 配置

首次运行会自动生成 `quickopen.toml`，格式如下：

```toml
[aliases]
ys = "\"D:/Games/miHoYo/miHoYo Launcher/launcher.exe\""
note = "notepad.exe"   # 系统 PATH 中的程序可以直接写名字
downloads = "D:/downloads"
github = "https://github.com"
rust_book = "https://doc.rust-lang.org/book/"
```

配置文件位置优先级：当前目录 `quickopen.toml` > `config.toml` > 用户目录 `.quickopen.toml`。

## 项目结构

```plain
src/
├── main.rs          # 入口，只负责解析参数和分发
├── config.rs        # 配置文件加载、保存、路径查找
├── launcher.rs      # 打开目录、URL、可执行文件（含提权）
├── alias.rs         # 别名的增删改查
└── utils.rs         # 辅助函数（环境变量展开、URL判断等）
```

## 用法

```bash
# 打开别名
qopen ys

# 传递参数
qopen note D:/test.txt

# 列出所有别名
qopen list

# 添加/更新别名
qopen add myapp D:/tools/app.exe

# 移除别名
qopen remove myapp
```

