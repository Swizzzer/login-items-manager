# Login Items Manager

![macOS](https://img.shields.io/badge/macOS-13%2B-000000?logo=apple&logoColor=white) ![Rust](https://img.shields.io/badge/Rust-2024-DEA584?logo=rust&logoColor=white) ![License](https://img.shields.io/badge/License-AGPL--3.0-blue)

> 一个基于 TUI 的 macOS 启动项管理器，读取系统登录项并支持交互式删除。

> [!WARNING]  
> 本项目未经过任何严格的测试，且删除启动项是一个具有一定风险的操作，在使用本工具时请务必三思而后行。
> 使用本工具造成的一切后果，Swizzer概不负责。

## 功能概览

- 💠 从 `sfltool dumpbtm` 解析系统登录项
- 🎫 虽然写得一坨但是姑且比直接`sfltool dumpbtm`更好看的TUI界面
- 🤓 支持直接删除部分启动项，无需在各个文件夹之间辗转(意思是有些启动项你还是得手动删除)

## 使用

```bash
cargo run --release
```

启动后会提示 sudo 授权，并进入终端界面。


## 快捷键

| 动作 | 按键 |
| --- | --- |
| 上下移动 | `↑/↓` 或 `j/k` |
| 跳到顶部/底部 | `g/G` 或 `Home/End` |
| 删除选中项 | `d` |
| 确认删除 | `y` 或 `Enter` |
| 取消删除 | `n` 或 `Esc` |
| 退出 | `q` |

