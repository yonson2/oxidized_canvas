# Config is layered: defaults.yaml plus per-environment overlays

Loco loads exactly one config file per environment (`{env}.local.yaml` or `{env}.yaml`, first match wins, no merging). We override `Hooks::load_config` in `app.rs` with a custom loader (`src/common/config.rs`) that deep-merges `config/defaults.yaml` with an optional `config/{env}.yaml` overlay — maps merge recursively, scalars and arrays are replaced by the overlay. Tera `get_env` templating is applied per file, mirroring Loco's render step. This makes `production.yaml` and `test.yaml` thin overrides instead of full copies of the development config. The override covers the CLI, tasks, examples, and the test harness, since all of them go through `Hooks::load_config`.

## Consequences

- `config/development.yaml` is gitignored: it is the local-override slot, not a committed file. A fresh clone boots development on `defaults.yaml` alone.
- Overlays cannot *remove* keys from defaults, only replace or add (e.g. tests now inherit the `static` middleware that the old standalone `test.yaml` omitted).
- Upgrading Loco means re-checking `Hooks::load_config` and `Config`'s deserialization requirements, since we bypass `Environment::load`.
