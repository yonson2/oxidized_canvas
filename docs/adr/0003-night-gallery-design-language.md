# The Gallery wears the Night Gallery design language

The Gallery's templates were re-skinned from Tailwind utility markup into a deliberate visual identity — the Night Gallery: warm near-black walls, a warm spotlight per artwork, Cormorant Garamond display type (self-hosted), a single verdigris accent, and the Placard as signature element. The stylesheet is one hand-written file (`assets/static/css/night-gallery.css`) with tokens named after the gallery's world (`--wall`, `--lamplight`, `--placard`, `--verdigris`); the Tailwind/npm pipeline was deleted since only the Gallery consumed it (the Backoffice carries its own styles). Pages intentionally have no navigation between them — the hidden-door quality is a design decision, not an oversight. Playful chrome (heart fireworks, animated glow borders, bouncing arrows) was deliberately quieted into still, verdigris-marked states.

## Considered Options

- **The White Cube** — museum-white walls, black type. The canonical gallery look, but dark-paletted art dies on white and it reads as the expected template.
- **The Salon** — parchment walls, dense classical framing, gold leaf. Characterful but fights the site's intentional minimalism; heavy frames compete with the art.
- **Keep Tailwind with a custom palette** — keeps the familiar toolchain, but two styling systems for five templates; utility-class markup is the default aesthetic being replaced.
- **Maud templates instead of Tera** — compile-time checked views were on the table; rejected as carrying no real benefit over Tera for a pure re-skin.

## Consequences

- CSS is an authored artifact with no build step; there is no `npm` toolchain left in the repo.
- Likes are acknowledged with a still verdigris mat-line instead of animated effects — the visual language assumes a quiet register everywhere.
- Future Gallery work should extend `night-gallery.css` tokens rather than introduce utility classes or new accent colors.
