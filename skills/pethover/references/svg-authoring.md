# Sticker SVG Authoring

**Read this when:** generating, reviewing, or validating `animation.svg` for a PetHover sticker pack.

Sticker SVGs are authored directly as XML by the LLM in one pass. They are never assembled by a Python, Node, SVG template, raster trace, frame extraction, GIF conversion, canvas renderer, or shape-string generator.

## Required root

The root element is:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 W H">
```

Absolute `width` and `height` attributes are forbidden. Sizing comes from `viewBox`.

Recommended view boxes:

- `192 208` for `slot="behind"` and `slot="over"`.
- `24 24` or `32 24` for `slot="corner"`.

## Allowed animation mechanisms

Use at least one of:

- SMIL elements: `<animate>`, `<animateTransform>`, `<animateMotion>`.
- CSS `@keyframes` inside an inline `<style>` element.

The SVG may use both mechanisms in the same file.

## Required reduced-motion fallback

Every sticker SVG includes this fallback in the inline `<style>` block:

```css
@media (prefers-reduced-motion: reduce) {
  * { animation: none !important; }
}
```

If the SVG uses only SMIL, still include the fallback. A future runtime may pause SMIL separately, but this media query is the shared convention with the existing emotion overlays in `src/styles.css`.

## Forbidden elements

The SVG must not contain:

- `<script>`
- `<foreignObject>`
- `<iframe>`
- `<image>`

## Forbidden references

The SVG must not contain any `href` or `xlink:href` value that starts with an external protocol, including:

- `http://`
- `https://`
- `ftp://`
- `data:image/`

Embedding raster data is forbidden.

## Font rule

External font references are forbidden. Use generic families such as `sans-serif` or `monospace`, or convert text to paths.

## Size cap

`animation.svg` must be 64 KB or smaller.

## Validation checklist

- XML is well formed.
- Root is `<svg>` with the SVG namespace and a `viewBox`.
- Root has no `width` or `height`.
- No forbidden element is present.
- No external `href` or `xlink:href` is present.
- At least one SMIL animation element or one CSS `@keyframes` rule is present.
- A `prefers-reduced-motion: reduce` media query is present.
- File size is 64 KB or smaller.
