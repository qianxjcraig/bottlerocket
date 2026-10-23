use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

// Upgrade leaves these unset, so the variant defaults turn on reclaim protection.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubernetes.kube-reserved-memory-min",
        "services.kube-reserved-memory-min",
        "configuration-files.runtime-memory-min-conf",
        "configuration-files.kubelet-memory-min-conf",
        "configuration-files.containerd-memory-min-conf",
    ]))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
