# PetHover

[English](./README.md)

桌面像素宠物，实时响应你的 AI Agent CLI 会话 —— Claude Code、Codex、Gemini、OpenCode。

基于 Tauri、Rust、React 构建。轻量、本地优先、无云依赖。

## 功能

- 实时响应 Agent 事件：思考、工具使用、等待输入、完成、错误。
- 内置 Claude Code、Codex、Gemini、OpenCode 适配器。
- 可自定义的像素宠物包（Codex 兼容格式）。
- 所有数据保存在 `~/.pethover`，无遥测、无云端。
- 安全的 CLI 配置改写：自动备份、原子写入、可卸载与修复。
- 双语界面（English / 简体中文）。

## 支持的 Agent

| Agent | 集成方式 | 默认配置路径 |
| --- | --- | --- |
| Claude Code | JSON hooks | `~/.claude/settings.json` |
| Codex | JSON hooks | `~/.codex/hooks.json` |
| Gemini | JSON hooks | `~/.gemini/settings.json` |
| OpenCode | JS 插件 | `~/.config/opencode/plugins/pethover.js` |

## 快速开始

环境要求：[Rust](https://www.rust-lang.org/tools/install)、[Node.js](https://nodejs.org/) 与 pnpm。支持 macOS（主要平台）、Windows、Linux。

```bash
git clone https://github.com/ChanceYu/pethover.git
cd pethover
pnpm install
pnpm tauri dev          # 开发模式
pnpm tauri build        # 构建发行版
```

## 项目结构

- `src-tauri/` — Rust 核心、Agent 适配器、运行时事件服务器。
- `src/` — React 前端（宠物窗口与设置中心）。
- `src-tauri/assets/pets/` — 应用打包的内置宠物包。
- `docs/architecture.zh.md` — 技术架构与设计文档。
- `AGENTS.md` — 贡献者指南与测试说明。

## 安全

- 事件服务器仅绑定 `127.0.0.1`，必须携带 bearer token，未知 payload 直接丢弃。
- 所有 hook 配置改写在写入前备份原始字节，并采用原子写。
- `assetProtocol.scope` 严格白名单 webview 可读的宠物资源目录。

## 贡献

欢迎 Issue 与 PR。先阅读 [AGENTS.md](AGENTS.md) 了解开发环境与约定，[docs/architecture.zh.md](docs/architecture.zh.md) 了解系统设计。

## 许可证

[MIT](LICENSE) © ChanceYu
