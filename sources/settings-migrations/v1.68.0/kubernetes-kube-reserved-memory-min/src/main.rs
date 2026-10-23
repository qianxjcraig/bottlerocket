use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

const SETTINGS: &[&str] = &[
    "settings.kubernetes.kube-reserved-memory-min.kubelet",
    "settings.kubernetes.kube-reserved-memory-min.containerd",
    "services.kube-reserved-memory-min.configuration-files",
    "services.kube-reserved-memory-min.restart-commands",
    "configuration-files.runtime-memory-min-conf.path",
    "configuration-files.runtime-memory-min-conf.template-path",
    "configuration-files.runtime-memory-min-conf.mode",
    "configuration-files.kubelet-memory-min-conf.path",
    "configuration-files.kubelet-memory-min-conf.template-path",
    "configuration-files.kubelet-memory-min-conf.mode",
    "configuration-files.containerd-memory-min-conf.path",
    "configuration-files.containerd-memory-min-conf.template-path",
    "configuration-files.containerd-memory-min-conf.mode",
];

// Upgrade leaves these settings unset so the new variant defaults apply. This
// intentionally enables reclaim protection. Downgrade removes all new keys.
fn run() -> Result<()> {
    migrate(AddSettingsMigration(SETTINGS))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration_helpers::{Migration, MigrationData};
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn release_manifest_references_compressed_migration() {
        // Image packaging compresses migration executables before placing them
        // in the migrations archive. Release.toml must name that exact target.
        let version = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let target = format!(
            "\"migrate_{version}_{}.lz4\"",
            env!("CARGO_PKG_NAME").trim_start_matches("migrate-")
        );
        assert!(
            include_str!("../../../../../Release.toml").contains(&target),
            "Release.toml does not reference the packaged migration {target}"
        );
    }

    fn existing() -> MigrationData {
        MigrationData {
            data: HashMap::from([
                (
                    "settings.kubernetes.kube-reserved.memory".into(),
                    json!("442Mi"),
                ),
                ("settings.kernel.sysctl.vm.swappiness".into(), json!("200")),
                (
                    "services.kubernetes.restart-commands".into(),
                    json!(["existing"]),
                ),
                (
                    "configuration-files.kubelet-exec-start-conf.path".into(),
                    json!("/existing"),
                ),
            ]),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn upgrade_leaves_new_settings_unset() {
        let input = existing();
        assert_eq!(
            AddSettingsMigration(SETTINGS)
                .forward(input.clone())
                .unwrap(),
            input
        );
    }

    #[test]
    fn downgrade_removes_settings_services_and_configuration_entries() {
        for quantities in [["128Mi", "128Mi"], ["200Mi", "300Mi"], ["128Mi", "0"]] {
            let mut input = existing();
            for key in SETTINGS {
                input.data.insert((*key).into(), json!("new"));
            }
            input.data.insert(SETTINGS[0].into(), json!(quantities[0]));
            input.data.insert(SETTINGS[1].into(), json!(quantities[1]));
            assert_eq!(
                AddSettingsMigration(SETTINGS).backward(input).unwrap(),
                existing()
            );
        }
    }

    #[test]
    fn downgrade_accepts_absent_settings() {
        assert_eq!(
            AddSettingsMigration(SETTINGS).backward(existing()).unwrap(),
            existing()
        );
    }
}
