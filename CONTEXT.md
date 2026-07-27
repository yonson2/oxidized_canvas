# Oxidized Canvas

Oxidized Canvas generates and publishes AI artwork: each Art piece gets an AI-generated image and title, and existing pieces can be combined into Mixes or re-rendered.

## Language

**Model**:
A specific AI model reachable through OpenRouter, identified by its OpenRouter model ID (e.g. `anthropic/claude-sonnet-4`). The unit of variation between generations.
_Avoid_: provider, service (when meaning what generates)

**Model Pool**:
The configured set of Models eligible for random selection for a kind of generation (text or image). Each generation flow (creating an Art, replacing one, mixing) draws one Model at random from the relevant pool and uses it for that flow's generations.
_Avoid_: provider list

**Provider**:
The single external gateway all AI generation goes through: OpenRouter. It never varies per call — variety comes from Models, not Providers. (Historically Anthropic, OpenAI, Google and BFL were separate Providers; that concept is gone.)

**The Gallery**:
The public-facing website (templates in `assets/views/` outside `backoffice/`): the latest-Art page, the infinite feed, and the Mix flow. It has no navigation between its pages on purpose — each page is reached by URL alone.
_Avoid_: main FE, frontend, public site

**Night Gallery**:
The Gallery's visual identity: warm near-black walls, each artwork under a subtle warm spotlight, serif display type (Cormorant Garamond), a single verdigris accent, sharp-edged artwork with a thin mat line.
_Avoid_: theme, skin, dark mode

**Placard**:
The small museum-style accession label accompanying each artwork: its number, title, medium and year, set in letter-spaced small caps. The signature element of the Night Gallery.
_Avoid_: caption, subtitle, metadata line
