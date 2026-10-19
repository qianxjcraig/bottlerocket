use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

const SETTINGS: &[&str] = &["settings.kubernetes.merge-default-eviction-settings"];

// Leave the new opt-in setting unset on upgrade; remove it on downgrade.
fn run() -> Result<()> {
    migrate(AddSettingsMigration(SETTINGS))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration_helpers::{Migration, MigrationData};
    use serde_json::json;
    use std::collections::HashMap;

    fn existing_settings() -> MigrationData {
        MigrationData {
            data: HashMap::from([
                (
                    "settings.kubernetes.eviction-hard.memory.available".into(),
                    json!("600Mi"),
                ),
                (
                    "settings.kubernetes.memory-swap-behavior".into(),
                    json!("LimitedSwap"),
                ),
            ]),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn upgrade_does_not_enable_merging() {
        let input = existing_settings();
        assert_eq!(
            AddSettingsMigration(SETTINGS)
                .forward(input.clone())
                .unwrap(),
            input
        );
    }

    #[test]
    fn downgrade_removes_only_the_new_setting() {
        for enabled in [true, false] {
            let expected = existing_settings();
            let mut input = expected.clone();
            input.data.insert(SETTINGS[0].into(), json!(enabled));
            assert_eq!(
                AddSettingsMigration(SETTINGS).backward(input).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn downgrade_accepts_an_unset_setting() {
        let input = existing_settings();
        assert_eq!(
            AddSettingsMigration(SETTINGS)
                .backward(input.clone())
                .unwrap(),
            input
        );
    }
}
