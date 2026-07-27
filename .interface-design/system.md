# Interface system — Night Gallery

Direction and rationale: `docs/adr/0003-night-gallery-design-language.md`.
Vocabulary (Gallery, Night Gallery, Placard): `CONTEXT.md`.
The stylesheet is the source of truth: `assets/static/css/night-gallery.css`. Extend it; do not add inline `<style>` blocks, utility classes, or a second styling system. There is no CSS build step.

## Direction and feel

A private gallery after closing time. Warm near-black walls, one pool of tungsten light per work, the artwork under a thin mat line, a museum placard beneath it. Chrome whispers; color lives in the art. The single accent is verdigris — never add a second. The Gallery has no navigation between its pages on purpose (hidden doors).

## Tokens

- Surfaces: `--wall #14100a` · `--wall-lift #1c1710` (raised: dropdowns) · `--wall-inset #0e0b06` (inputs are recessed)
- Light: `--lamplight: 255, 213, 154` — always used at low alpha in the `.spotlight` radial gradient (0.12)
- Ink: `--ink #ece4d3` · `--ink-secondary #a89d87` · `--ink-muted #6e6553`
- Accent: `--verdigris #5fb3a1`, hover `--verdigris-bright #82cfbd`
- Lines: `--mat-line rgba(236,228,211,.16)`, emphasis `--mat-line-strong (.34)`
- Depth: one motivated shadow only — `--shadow-canvas` makes the artwork float off the wall. No other shadows; hairline borders everywhere else.
- Radius: 0. Sharp edges, always.

## Type

- Display: `Cormorant Garamond` 400 upright + italic, self-hosted in `assets/static/fonts/` (latin + latin-ext, declared with unicode-ranges, preloaded in page heads). Used for `.wordmark` (italic), `.work-title`, `.studio-prompt`.
- Label: system sans in small caps — `font-size: 0.6875rem; letter-spacing: 0.18em; text-transform: uppercase` (placards, nav, status, buttons at 0.75rem/0.22em).
- Body/system text falls back to `--font-label` at `font-size: 0.75rem`.

## Spacing

Base 4px. Figure gap 1.25rem, form gap 1.5rem, room padding 1rem, page bottom padding 3rem. Content column max-width 34rem.

## Component patterns

- `.spotlight` — `::before` radial lamplight gradient; one per room/feed item. Room pages put it on `.gallery-room` (not `<main>`) so the pool spans the header without a seam.
- `.frame` — mat-line border + `--shadow-canvas` + inset background. `.frame.liked::after` is the collector's mark: a still verdigris line inset 7px (animation `collectorMark`, 500ms once). Liked works never animate in a loop.
- `.placard` — accession label; `<span class="accession">#N</span> · Diffusion on canvas · YYYY`. Mixes use `#M{n}`.
- `.gallery-nav` — prev/next room links, uppercase label style, verdigris on hover only.
- Feed structure: `.snap-container` > `.snap-item.spotlight[data-image-id]` > `.work-figure` (title / `.frame.image-container[data-image-id]` / placard). JS depends on the class names `snap-item`, `image-container`, `like-button` and on `data-image-id` — keep them.
- `.like-button` — overlay heart, `opacity: 0` at rest; `animating.liking|.unliking` plays `heartPulse` 0.8s once (verdigris fill on like). No fireworks, no particle effects.
- `.scroll-hint` — still hairline chevron; JS only fades it via `opacity`.
- Studio (mix): `.studio-prompt`, `.studio-form` (`.form-disabled` dims + disarms), `.mix-button` (filled verdigris, dark text), `.mix-loader` (1px verdigris line filling over 90s), `.mix-status` (`.error` → verdigris-bright).
- choices.js: its stylesheet loads after ours — skin rules must carry the extra `.choices` class (`.choices .choices__inner`) to win the cascade. Prefix every new override the same way.

## Register

Interactions get one quiet acknowledgment, then stillness. No loops, no bounce, no springy easing. Transitions 200–400ms ease; one-shot animations ≤ 0.8s ease-out.
