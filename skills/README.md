# CoPet Skills

CoPet-specific skill packages. Each skill documents one global CoPet pack type and is installable on its own.

Today there are two CoPet skills:

- [`copet-sound`](./copet-sound/SKILL.md) creates global 11-clip MP3 sound packs.
- [`copet-gen`](./copet-gen/SKILL.md) creates CoPet pet packages by delegating generation to `$hatch-pet`, allowing its subagents, and installing the result.

Only `copet-gen` installs pet spritesheets and `pet.json`; it does so by requiring the upstream `$hatch-pet` skill to generate and validate the source package first.

## Global Package Layout

```text
$HOME/.copet/
├── pets/
│   └── <pet-id>/
│       ├── pet.json
│       └── spritesheet.webp
├── sounds/
│   └── <sound-pack-id>/
│       ├── sound.json
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
```

Pack ids are kebab-case slugs derived from `displayName`. If a slug collides in its target global directory, append `-2`, `-3`, and continue until the destination is unique.

## The Skills

| Folder | `name` | `displayName` | Owns |
|---|---|---|---|
| [`copet-sound/`](./copet-sound/SKILL.md) | `copet-sound` | CoPet Sound | `$HOME/.copet/sounds/<sound-pack-id>/`, `sound.json`, and the 11 required MP3 clips. |
| [`copet-gen/`](./copet-gen/SKILL.md) | `copet-gen` | CoPet Gen | `$HOME/.copet/pets/<pet-id>/`, `pet.json`, and `spritesheet.webp` copied from a completed `$hatch-pet` package. |

## Single-Responsibility Policy

Each skill folder is self-contained. No file inside a skill folder may link to files outside its own folder. A pack author or runtime implementer only needs to read the skill for the domain they are working on; installing either skill in isolation must give complete documentation for that pack type.

Outbound references may be sibling-skill references such as `$hatch-pet` or public URLs, but each such reference must document a public-URL fallback so consumers can install the dependency if it is not present locally.

The skills here are documentation artifacts. They describe pack formats and runtime contracts, not executable application code.
