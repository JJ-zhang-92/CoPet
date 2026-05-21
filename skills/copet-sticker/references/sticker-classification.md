# Sticker Classification

**Read this when:** deciding a sticker's `kind`, `slot`, and state binding from user input.

When input is unambiguous, classify with this table. When input is ambiguous, ask one clarifying question instead of guessing.

Sticker classification chooses behavior for a decoration/effect layer. It never authorizes drawing a standalone pet, animal, mascot, character body, head, face, or silhouette.

## Legal values

`kind`:

- `persistent`
- `burst`

`slot`:

- `behind`
- `over`
- `corner`

Pet states:

- `idle`
- `running-right`
- `running-left`
- `waving`
- `jumping`
- `failed`
- `waiting`
- `running`
- `review`

Agent state kinds:

- `none`
- `thinking`
- `editing`
- `inspecting`
- `awaitingApproval`
- `celebrating`
- `hurt`

Emotion state kinds:

- `none`
- `loadingBubble`
- `sparkle`
- `smoke`
- `heart`
- `questionMark`

Do not use `none` as the only binding for a burst trigger. A trigger must identify a visible runtime event.

## Classification table

| Input signal | `kind` | `slot` | Binding |
|---|---|---|---|
| Celebration or burst: `烟花`, `欢呼`, `礼花`, `fireworks`, `confetti`, `cheer` | `burst` | `over` | `trigger.agentStates=["celebrating"]` or `trigger.states=["waving"]` |
| Mood bubble: `心心`, `?`, `...`, `汗滴`, `thought`, `question`, `heart` | `burst` | `corner` | `trigger.emotions` with the matching emotion kind |
| Environmental ambience: `雪`, `雨`, `星空`, `极光`, `彩虹`, `snow`, `rain`, `aurora`, `rainbow` | `persistent` | `behind` | `visibility.states` contains all nine pet states |
| Aura or power buff: `光环`, `气场`, `燃烧`, `halo`, `aura`, `power` | `persistent` | `behind` | `visibility.states=["idle","waiting","review"]` |
| Floating particles above the pet: `音符`, `星粒`, `气泡`, `notes`, `sparkles`, `bubbles` | `persistent` | `over` | `visibility.states` contains all nine pet states |

## Subject normalization

If the input names an animal, pet, person, mascot, or character, do not classify it as something to depict. Normalize it into decorative motifs before selecting the row:

- Cat, dog, fox, panda, or other animal -> paw prints, whisker-like swooshes, color accents, sparkles, tiny symbols.
- Robot, ghost, hero, or humanoid character -> circuit sparks, aura shapes, icons, smoke, glow, punctuation, particles.
- A reference image -> palette, mood, texture, and motion cues only.

## Ambiguity rule

If the input could fit multiple rows and the desired behavior is unclear, ask exactly one clarifying question in the response language. The question must ask whether the sticker should be a one-shot burst for a specific moment or a persistent decoration that stays visible across states. Do not reuse English wording for non-English requests.

After the user answers, classify with the table.

## Persistent all-state note

The live `sticker.json` schema requires `visibility` for every `kind="persistent"` sticker. To represent "all states", set:

```json
{
  "visibility": {
    "states": [
      "idle",
      "running-right",
      "running-left",
      "waving",
      "jumping",
      "failed",
      "waiting",
      "running",
      "review"
    ]
  }
}
```
