---
name: pethover
description: Use when generating PetHover global audio packs or animated sticker packs from a user image or text description.
---

# PetHover

## Overview

This skill is the single orchestration entry point for PetHover global pack generation. It produces one of two self-contained pack types:

1. **audios** - an 11-clip MP3 audio pack under `$HOME/.pethover/audios/<audio-pack-id>/`.
2. **stickers** - an animated SVG sticker pack under `$HOME/.pethover/stickers/<sticker-id>/`.

This skill does not generate sprite atlases, omni directional atlases, or pet packages. It never writes to `$HOME/.pethover/pets/`, never creates an empty pet package, and never modifies `pet.json`. If the caller wants a fresh Codex-compatible pet spritesheet, invoke `$hatch-pet` directly instead of this skill.

Every shipped artifact must come from the required authoring backend:

- Audio MP3s come from a real audio-generation backend, text-to-speech backend, sound-effect generation backend, field recording library, or curated sample library.
- Sticker SVGs are emitted directly by the LLM as XML in a single authoring pass.

Procedural substitutes are failed runs even when structural validation passes.

## Global package layout

```text
$HOME/.pethover/
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

Pack ids are kebab-case slugs derived from `displayName`. If a slug collides with an existing entry under the same global directory, append `-2`, `-3`, and continue until the destination is unique.

## Inputs

| Input kind | Format | Notes |
|---|---|---|
| `image` | PNG or JPEG, 8 MB or smaller | A reference picture. It must be decodable and not transparent-only. |
| `text` | UTF-8 string, 2,000 characters or fewer | A description of the desired audio character or sticker. It must not be empty after trimming whitespace. |

Exactly one primary input kind is used. An image with a caption is allowed; the image is primary and the caption is supporting context.

Reject invalid input before presenting the menu or invoking any sub-task.

## Menu

Present a single-choice menu:

```text
Which PetHover pack should I generate?
  ○ audios   - global 11-clip MP3 pack
  ○ stickers - global animated SVG sticker pack
```

Accept exactly one choice. Reject multi-select with:

```text
Choose exactly one PetHover pack type: audios or stickers.
```

Reject zero selections with:

```text
Choose a PetHover pack type before generation: audios or stickers.
```

## Pipeline

The pipeline has five steps: validate input -> choose one sub-task -> stage -> execute -> validate and promote.

### 1. Validate input

- Image: decodable, 8 MB or smaller, not transparent-only.
- Text: non-empty after trimming, 2,000 characters or fewer.

Reject invalid input before generation.

### 2. Choose one sub-task

Ask the single-choice menu above unless the caller already explicitly selected `audios` or `stickers`.

Do not ask for additional profile choices. Use the defaults documented in the selected sub-task reference.

### 3. Stage

Create one empty staging directory:

```text
$HOME/.pethover/tmp/audios-<unix-epoch>-<audio-pack-id>/
$HOME/.pethover/tmp/stickers-<unix-epoch>-<sticker-id>/
```

Every file written during generation goes into that staging directory. The live `$HOME/.pethover/audios/` and `$HOME/.pethover/stickers/` directories are read-only until final promotion.

### 4. Execute selected sub-task

For `audios`, read `references/sub-task-audio.md`.

The audio sub-task:

1. Infers animal class, object class, material, size, energy, and voice character from the raw user input.
2. Derives `displayName`, `displayNameZh`, and `id`.
3. Writes 11 real MP3 clips into staging.
4. Composes `audio-pack.json` in the staging root.

For `stickers`, read `references/sub-task-sticker.md`.

The sticker sub-task:

1. Classifies `kind`, `slot`, and `trigger` or `visibility` with `references/sticker-classification.md`.
2. Derives `displayName`, `displayNameZh`, and `id`.
3. Emits `animation.svg` directly as SVG XML.
4. Composes `sticker.json` in the staging root.

### 5. Validate and atomically promote

Validate the full staging directory before promotion.

For audio packs, use `references/audio-pack-schema.md`.

For sticker packs, use `references/sub-task-sticker.md` and `references/svg-authoring.md`.

On validation success, atomically rename staging into the final live directory:

```text
$HOME/.pethover/audios/<audio-pack-id>/
$HOME/.pethover/stickers/<sticker-id>/
```

On validation failure, leave staging in place, report the specific failed checklist item, and do not touch the live directory.

## Generation discipline

Audio generation must not fall back to synthesized tones, oscillator output, MIDI rendering, silence, or pitch-shifted duplicates. The output must be authored audio from a real backend or curated source.

Sticker generation must not fall back to procedural SVG assembly, programmatic path construction, raster tracing, GIF conversion, canvas rendering, or embedded images. The LLM authors the SVG XML directly.

Image input is only a reference for palette, character mood, or subject cues. It is never embedded into sticker SVG and never passed directly to an audio backend as a waveform source.

If the selected sub-task cannot satisfy its generation discipline after three attempts, abort the run, leave staging in place, and report the specific missing backend or validation failure.

## Anti-patterns

Read `references/anti-patterns.md` before generation and when diagnosing a failure.

## References

- `references/sub-task-audio.md` - audio pack generation.
- `references/sub-task-sticker.md` - sticker pack generation.
- `references/audio-pack-schema.md` - `audio-pack.json` schema and validation.
- `references/svg-authoring.md` - sticker SVG authoring rules.
- `references/sticker-classification.md` - sticker kind, slot, and binding classification.
- `references/anti-patterns.md` - hard failure rules.
- `references/audio-asset-format.md` - MP3 format guidance.
- `references/gesture-sound-map.md` - advisory interaction sound roles.
