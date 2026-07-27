use std::{
    fs,
    path::{Path, PathBuf},
};

use loco_rs::{Error, Result, config::Config, environment::Environment};

const DEFAULT_CONFIG_FOLDER: &str = "config";
const CONFIG_FOLDER_ENV: &str = "LOCO_CONFIG_FOLDER";

/// `load` resolves the app's configuration as `defaults.yaml` deep-merged with
/// the optional `{environment}.yaml` overlay, with Tera `get_env` templating
/// applied to each file.
///
/// # Errors
///
/// If `defaults.yaml` is missing, a template fails to render (e.g. a `get_env`
/// without default whose variable is unset), or the merged YAML doesn't match
/// the expected configuration shape.
pub fn load(env: &Environment) -> Result<Config> {
    let folder = std::env::var_os(CONFIG_FOLDER_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FOLDER));
    load_from_folder(env, &folder)
}

fn load_from_folder(env: &Environment, folder: &Path) -> Result<Config> {
    let defaults_path = folder.join("defaults.yaml");
    let defaults = fs::read_to_string(&defaults_path).map_err(|e| {
        Error::Message(format!("could not read {}: {e}", defaults_path.display()))
    })?;

    let overlay_path = folder.join(format!("{env}.yaml"));
    let overlay = fs::read_to_string(&overlay_path).ok();

    let mut merged = parse_rendered(&defaults, &defaults_path)?;
    if let Some(overlay) = overlay {
        merged = merge_values(merged, parse_rendered(&overlay, &overlay_path)?);
    }

    serde_yaml::from_value(merged)
        .map_err(|e| Error::YAMLFile(e, defaults_path.to_string_lossy().to_string()))
}

fn parse_rendered(content: &str, path: &Path) -> Result<serde_yaml::Value> {
    let rendered = tera::Tera::one_off(content, &tera::Context::new(), false)
        .map_err(|e| Error::Message(format!("Failed to render {}: {e}", path.display())))?;
    serde_yaml::from_str(&rendered)
        .map_err(|e| Error::YAMLFile(e, path.to_string_lossy().to_string()))
}

fn merge_values(base: serde_yaml::Value, overlay: serde_yaml::Value) -> serde_yaml::Value {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(mut base), serde_yaml::Value::Mapping(overlay)) => {
            for (key, overlay_value) in overlay {
                let merged = match base.remove(&key) {
                    Some(base_value) => merge_values(base_value, overlay_value),
                    None => overlay_value,
                };
                base.insert(key, merged);
            }
            serde_yaml::Value::Mapping(base)
        }
        (_, overlay) => overlay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, str::FromStr};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "oxidized_canvas_config_test_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    const DEFAULTS: &str = r"
logger:
  enable: true
  level: debug
  format: compact
server:
  binding: 0.0.0.0
  port: 5150
  host: http://localhost
database:
  uri: sqlite://base.sqlite?mode=rwc
  enable_logging: false
  connect_timeout: 500
  idle_timeout: 500
  min_connections: 1
  max_connections: 1
  auto_migrate: true
  dangerously_truncate: false
  dangerously_recreate: false
settings:
  openrouter_api_key: placeholder
  text_models:
    - base/model
";

    #[test]
    fn merge_overlay_wins_scalars() {
        let base: serde_yaml::Value = serde_yaml::from_str("a: 1").unwrap();
        let overlay: serde_yaml::Value = serde_yaml::from_str("a: 2").unwrap();
        let merged = merge_values(base, overlay);
        assert_eq!(merged["a"], serde_yaml::Value::from(2));
    }

    #[test]
    fn merge_deep_merges_maps_keeping_base_keys() {
        let base: serde_yaml::Value = serde_yaml::from_str("settings:\n  key: base\n  pool:\n    - m1\n").unwrap();
        let overlay: serde_yaml::Value = serde_yaml::from_str("settings:\n  key: overlay\n").unwrap();
        let merged = merge_values(base, overlay);
        assert_eq!(merged["settings"]["key"], serde_yaml::Value::from("overlay"));
        assert_eq!(
            merged["settings"]["pool"],
            serde_yaml::Value::from(vec![serde_yaml::Value::from("m1")])
        );
    }

    #[test]
    fn merge_arrays_are_replaced_not_appended() {
        let base: serde_yaml::Value = serde_yaml::from_str("pool:\n  - a\n  - b\n").unwrap();
        let overlay: serde_yaml::Value = serde_yaml::from_str("pool:\n  - c\n").unwrap();
        let merged = merge_values(base, overlay);
        assert_eq!(
            merged["pool"],
            serde_yaml::Value::from(vec![serde_yaml::Value::from("c")])
        );
    }

    #[test]
    fn loads_defaults_when_no_overlay_exists() {
        let dir = temp_dir("defaults_only");
        fs::write(dir.join("defaults.yaml"), DEFAULTS).unwrap();

        let config =
            load_from_folder(&Environment::from_str("development").unwrap(), &dir).unwrap();
        assert_eq!(config.server.port, 5150);
        assert_eq!(config.database.uri, "sqlite://base.sqlite?mode=rwc");
    }

    #[test]
    fn overlay_overrides_and_inherits() {
        let dir = temp_dir("overlay");
        fs::write(dir.join("defaults.yaml"), DEFAULTS).unwrap();
        fs::write(
            dir.join("development.yaml"),
            "server:\n  port: 9999\nsettings:\n  openrouter_api_key: sk-or-real\n",
        )
        .unwrap();

        let config =
            load_from_folder(&Environment::from_str("development").unwrap(), &dir).unwrap();
        // overridden by overlay
        assert_eq!(config.server.port, 9999);
        // inherited from defaults
        assert_eq!(config.database.uri, "sqlite://base.sqlite?mode=rwc");
        let settings = config.settings.unwrap();
        assert_eq!(settings["openrouter_api_key"], "sk-or-real");
        assert_eq!(settings["text_models"][0], "base/model");
    }

    #[test]
    fn errors_when_defaults_missing() {
        let dir = temp_dir("missing_defaults");
        let result = load_from_folder(&Environment::from_str("development").unwrap(), &dir);
        assert!(result.is_err());
    }

    #[test]
    fn renders_get_env_templating() {
        const VAR: &str = "OXIDIZED_CANVAS_CONFIG_TEST_VAR";
        unsafe { std::env::set_var(VAR, "from-env") };

        let dir = temp_dir("tera");
        fs::write(
            dir.join("defaults.yaml"),
            DEFAULTS.replace("placeholder", &format!("{{{{ get_env(name=\"{VAR}\") }}}}")),
        )
        .unwrap();

        let config =
            load_from_folder(&Environment::from_str("development").unwrap(), &dir).unwrap();
        let settings = config.settings.unwrap();
        assert_eq!(settings["openrouter_api_key"], "from-env");

        unsafe { std::env::remove_var(VAR) };
    }
}
