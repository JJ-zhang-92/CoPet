# PetHover audio sub-task (3b)

**Read this when:** the user selected **PetHover audio** in step 2 — you are about to infer the animal class from the raw user input and synthesize the 11 MP3 clips.

## Independence from sprite

This sub-task runs from the **raw user input** (image + caption, or text-only description) and is **independent** of the sprite sub-task. It must not wait for `$hatch-pet` to finish, and it must not read any sprite-produced metadata. If both audio and sprite are selected in the same run, they start together and consume the same raw input independently — the merge in step 4 reconciles them afterwards.

## Abort if the real backend is unavailable

This sub-task **only** ships MP3 clips produced by a real audio-generation backend — text-to-speech, sound-effect generation, or a curated sample library matched to the inferred (category, traits) tuple. See SKILL.md ["Generation backend discipline"](../SKILL.md#generation-backend-discipline) for the rationale.

If no such backend is available in the current environment, **abort the audio sub-task and surface the missing-backend error.** Per step 3's partial-failure rule, that aborts the whole run; do not "partially succeed" by shipping placeholder audio.

**Specifically forbidden substitutes** (each one is a generation error):

- Code-synthesized waveforms via `ffmpeg sine=` / `aevalsrc=` / `tremolo=` / pure-tone generation / FM synthesis / MIDI rendering / any oscillator-driven approach, even when wrapped in a valid MP3 container at the correct sample rate, mono channel, loudness, and ≤ 1-second duration.
- Drawing a "vocal envelope" in code (attack / sustain / release on a synthesized tone) and labeling it "click", "purr", "celebrating", etc.
- Using the same generated waveform across multiple clip keys with minor pitch or duration tweaks.
- Authoring an MP3 of silence (or near-silence) and accepting it because the audio-format validation in step 5 only checks container, bit-rate, and size — it cannot tell whether the audio is meaningful.

The audio-format validation in step 5 (`.mp3` container, ≤ 16 MB, within the loudness target) is structural; it cannot detect that the file contains a sine sweep instead of a real bark. The responsibility for refusing synthesized tones sits here, in the generation step.

## What this sub-task owns

Only `pethover.audio`. Nothing else.

## Inferring the audio target from input

Before synthesizing clips, derive the *audio character* from the user input:

### From an image

Classify the depicted subject. Identify the broadest applicable category (dog, cat, bird, fox, dragon, robot, blob, plush, etc.) and any obvious traits (small/large, fluffy/sleek, energetic/calm, fantasy/realistic). When the image is ambiguous (stylized, mythical, or hybrid), choose the closest real-world vocal analogue — e.g. small mythical creature → a small mammal's vocal palette — and document the choice in the run log.

### From text

Parse the description for:

- **Explicit species** ("a corgi", "a robot cat", "a phoenix")
- **Personality cues** ("grumpy", "playful", "regal")
- **Size and age cues** ("a tiny kitten", "an old wizard cat")

If the text gives only personality, default the vocal class to a small mammal unless something else is clearly indicated.

### How the derived (category, traits) tuple drives synthesis

- **Vocal palette**: bark / meow / chirp / synth tones / wordless coos / mechanical clicks / etc.
- **Pitch range and articulation**: an energetic small dog gets short, bright clicks; a calm large bear gets low, soft rumbles.
- **Mood split**: interaction clips skew sharper and reactive; agent clips skew smoother and more ambient.

## Clip set (11 clips total)

A full PetHover audio set is the 11 clips below. Omit keys that are not generated.

### Interaction sounds (5)

| Key | When it plays |
|---|---|
| `click` | Single user click |
| `doubleClick` | Two clicks within the double-click window |
| `petted` | Rapid repeated clicks |
| `pettedSlow` | Sustained long-press |
| `dragLand` | Pet dropped after a drag |

### Agent-state sounds (6)

| Key | When it plays |
|---|---|
| `celebrating` | Agent finished a task |
| `failed` | Agent task failed |
| `thinking` | Agent reasoning / planning |
| `editing` | Agent writing code |
| `inspecting` | Agent reading code |
| `awaitingApproval` | Agent waiting on user |

See [`gesture-sound-map.md`](./gesture-sound-map.md) for suggested sound *roles* (advisory) and [`audio-asset-format.md`](./audio-asset-format.md) for binding asset rules (MP3 only, size cap, loudness target, silence trimming).

Save each generated clip under `<staging-dir>/pethover/audio/` — where `<staging-dir> = $HOME/.pethover/tmp/pet-<unix-epoch>-<pet-id>/` is the per-run staging directory set up at the start of step 3. **Never** write directly under `$HOME/.pethover/pets/<pet-id>/`; that location is read-only until the atomic promotion at the end of step 5.

Filenames are free-form; the manifest fragment references them by relative path under `pethover/audio/...`. Do **not** add a manifest key for a missing or failed clip.

## Audio manifest fragment

Emit only `pethover.audio`:

```json
{
  "pethover": {
    "audio": {
      "interactionSounds": { "click": "pethover/audio/click.mp3", "...": "..." },
      "agentSounds":       { "thinking": "pethover/audio/hmm.mp3", "...": "..." }
    }
  }
}
```

Hold this fragment in memory. The merge step (4) will apply it to `pet.json`. Do **not** write `pet.json` from inside this sub-task.
