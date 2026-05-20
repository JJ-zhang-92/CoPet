# Anti-Patterns

**Read this when:** before committing to a generation step, or when a run is failing and you need to check whether a hard rule was violated.

Each "Don't" is a hard rule. Violating it is a failed run.

## Menu and pipeline

- Don't run generation before validating the input.
- Don't accept multiple menu choices. This skill runs exactly one of `audios` or `stickers`.
- Don't accept zero menu choices.
- Don't generate sprite atlases from this skill. Invoke `$hatch-pet` directly for sprite work.
- Don't generate omni directional body atlases from this skill.
- Don't create, copy, seed, or modify any pet package.
- Don't read or write `$HOME/.pethover/pets/`.
- Don't write or modify `pet.json`.
- Don't write into the live `$HOME/.pethover/audios/` or `$HOME/.pethover/stickers/` directory before validation passes.
- Don't promote staging via copy-and-delete. Promotion is a directory rename.
- Don't delete staging on failure. Leave it available for debugging.

## Audio packs

- Don't ask for a target pet. Audio packs are global.
- Don't omit any of the 11 required MP3 clips.
- Don't add extra top-level keys to `audio-pack.json`.
- Don't reference paths outside the audio pack root.
- Don't use nested paths for MP3 files. The v1 pack layout keeps all 11 clips beside `audio-pack.json`.
- Don't synthesize audio from oscillators, generated tones, MIDI, `ffmpeg sine=`, `aevalsrc=`, `tremolo=`, or code-generated waveforms.
- Don't reuse one clip across several keys with pitch, speed, or duration tweaks.
- Don't ship silence or near-silence as a valid clip.
- Don't exceed 16 MB per MP3 file.

## Sticker packs

- Don't guess when the input could be either burst or persistent. Ask one clarifying question.
- Don't omit `trigger` from a burst sticker.
- Don't include `visibility` on a burst sticker.
- Don't omit `visibility` from a persistent sticker.
- Don't include `trigger` on a persistent sticker.
- Don't use state names outside the current `PetStateId`, `AgentState`, or `EmotionState` unions.
- Don't embed uploaded images into SVG.
- Don't use `<image>`, `<script>`, `<foreignObject>`, or `<iframe>`.
- Don't use external `href`, `xlink:href`, external fonts, URLs, or `data:image/` references.
- Don't set root `width` or `height`; use `viewBox`.
- Don't omit the `prefers-reduced-motion: reduce` fallback.
- Don't ship a non-animated SVG. Use SMIL or CSS keyframes.
- Don't exceed 64 KB for `animation.svg`.
- Don't assemble sticker SVGs with Python, Node, templates, generated path strings, canvas, raster tracing, GIF conversion, or frame extraction.

## Cleanliness

- Don't leave `.tmp`, `.bak`, `.swp`, `.DS_Store`, source prompts, preview files, scratch directories, `.hatch-run`, or `.hatch-codex` in staging.
- Don't leave files not referenced by the manifest in staging.
- Don't promote a staging directory unless it contains exactly the files required by the selected pack type.
