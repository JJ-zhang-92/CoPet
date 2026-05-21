# HoverPet Skills

HoverPet-specific skill packages. Each skill documents one global HoverPet pack type and is installable on its own.

Today there are two HoverPet skills:

- [`hoverpet-audio`](./hoverpet-audio/SKILL.md) creates global 11-clip MP3 audio packs.
- [`hoverpet-sticker`](./hoverpet-sticker/SKILL.md) creates decorative animated SVG sticker packs.

These skills do not generate pet spritesheets, omni directional body atlases, pet packages, or `pet.json`. For a fresh Codex-compatible pet spritesheet, use the upstream `$hatch-pet` skill instead.

## Global Package Layout

```text
$HOME/.hoverpet/
├── audios/
│   └── <audio-pack-id>/
│       ├── audio-pack.json
│       ├── click.mp3
│       ├── surprised.mp3
│       ├── purr.mp3
│       ├── sigh.mp3
│       ├── wheee.mp3
│       ├── hmm.mp3
│       ├── tap.mp3
│       ├── peek.mp3
│       ├── wait.mp3
│       ├── yay.mp3
│       └── oof.mp3
└── stickers/
    └── <sticker-id>/
        ├── sticker.json
        └── animation.svg
```

Pack ids are kebab-case slugs derived from `displayName`. If a slug collides in its target global directory, append `-2`, `-3`, and continue until the destination is unique.

## The Skills

| Folder | `name` | `displayName` | Owns |
|---|---|---|---|
| [`hoverpet-audio/`](./hoverpet-audio/SKILL.md) | `hoverpet-audio` | HoverPet Audio | `$HOME/.hoverpet/audios/<audio-pack-id>/`, `audio-pack.json`, and the 11 required MP3 clips. |
| [`hoverpet-sticker/`](./hoverpet-sticker/SKILL.md) | `hoverpet-sticker` | HoverPet Sticker | `$HOME/.hoverpet/stickers/<sticker-id>/`, `sticker.json`, and `animation.svg`. |

## Single-Responsibility Policy

Each skill folder is self-contained. No file inside a skill folder may link to files outside its own folder. A pack author or runtime implementer only needs to read the skill for the domain they are working on; installing either skill in isolation must give complete documentation for that pack type.

Outbound references may be sibling-skill references such as `$hatch-pet` or public URLs, but each such reference must document a public-URL fallback so consumers can install the dependency if it is not present locally.

The skills here are documentation artifacts. They describe pack formats and runtime contracts, not executable application code.
