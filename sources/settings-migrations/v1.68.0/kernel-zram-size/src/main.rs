use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

const SETTINGS: &[&str] = &[
    "settings.kernel.zram.size",
    "services.zram.configuration-files",
    "services.zram.restart-commands",
];

// Preserve the existing 1 GiB behavior on upgrade; remove the new setting and
// configuration service before downgrading to a version without support.
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

    fn existing() -> MigrationData {
        MigrationData {
            data: HashMap::from([
                ("settings.kernel.sysctl.vm.swappiness".into(), json!("200")),
                (
                    "settings.kubernetes.memory-swap-behavior".into(),
                    json!("NoSwap"),
                ),
            ]),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn upgrade_keeps_the_setting_unset() {
        let input = existing();
        assert_eq!(
            AddSettingsMigration(SETTINGS)
                .forward(input.clone())
                .unwrap(),
            input
        );
    }

    #[test]
    fn downgrade_removes_only_zram_setting_and_service() {
        for size in ["512Mi", "2Gi", "25%", "0", "0%"] {
            let expected = existing();
            let mut input = expected.clone();
            input.data.insert(SETTINGS[0].into(), json!(size));
            input
                .data
                .insert(SETTINGS[1].into(), json!(["corndog-toml"]));
            input.data.insert(SETTINGS[2].into(), json!([]));
            assert_eq!(
                AddSettingsMigration(SETTINGS).backward(input).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn downgrade_accepts_unset() {
        let input = existing();
        assert_eq!(
            AddSettingsMigration(SETTINGS)
                .backward(input.clone())
                .unwrap(),
            input
        );
    }
}
