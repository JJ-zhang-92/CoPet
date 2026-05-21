# HoverPet

[简体中文](./README.zh.md)

A pixel-art desktop pet that reacts in real time to your AI Agent CLI sessions — Claude Code, Codex, Gemini, OpenCode.

Built with Tauri, Rust, and React. Lightweight, local-first, no cloud.

## Features

- Real-time pet reactions to Agent events: thinking, tool use, waiting, completion, errors.
- Adapters for Claude Code, Codex, Gemini, and OpenCode.
- Customizable pixel-art pet packages (Codex-compatible format).
- All data stays in `~/.hoverpet` — no telemetry, no cloud.
- Safe CLI config edits: automatic backup, atomic writes, easy uninstall/repair.
- Bilingual UI (English / 简体中文).

## Supported agents

| Agent | Integration | Default config path |
| --- | --- | --- |
| Claude Code | JSON hooks | `~/.claude/settings.json` |
| Codex | JSON hooks | `~/.codex/hooks.json` |
| Gemini | JSON hooks | `~/.gemini/settings.json` |
| OpenCode | JS plugin | `~/.config/opencode/plugins/hoverpet.js` |

## Getting started

Prerequisites: [Rust](https://www.rust-lang.org/tools/install), [Node.js](https://nodejs.org/) with pnpm. Runs on macOS (primary), Windows, and Linux.

```bash
git clone https://github.com/ChanceYu/HoverPet.git
cd HoverPet
pnpm install
pnpm tauri dev          # development
pnpm tauri build        # production bundle
```

## Project layout

- `src-tauri/` — Rust core, agent adapters, runtime server.
- `src/` — React frontend (pet window + settings center).
- `src-tauri/assets/pets/` — built-in pet packages bundled with the app.
- `docs/architecture.md` — technical architecture and design.
- `AGENTS.md` — contributor guide and testing instructions.

## Security

- Event server binds only to `127.0.0.1`, requires a bearer token, drops unknown payloads.
- All hook config changes are backed up before write and use atomic file ops.
- `assetProtocol.scope` whitelists exactly which pet directories the webview can read.

## Contributing

Issues and PRs welcome. Start with [AGENTS.md](AGENTS.md) for setup and conventions, and [docs/architecture.md](docs/architecture.md) for the system design.

## License

[MIT](LICENSE) © ChanceYu
