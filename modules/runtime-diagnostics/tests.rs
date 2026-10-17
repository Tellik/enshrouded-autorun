use super::*;
use serde_json::json;
use std::time::{Duration, Instant};
#[test]
fn disabled_means_no_measurements_and_snapshot_finishes() {
    let root = tempfile::tempdir().unwrap();
    let mut session = Session::new(root.path());
    session.measure(
        "mods",
        "example",
        "on_update",
        Duration::from_millis(30),
        false,
    );
    assert!(session.measurements.is_empty());
    request(root.path(), "snapshot").unwrap();
    session.next_config = Instant::now();
    session.tick();
    assert!(!session.active);
    assert_eq!(session.reason, "snapshot-complete");
    assert!(session.last_sample > 0);
    assert_eq!(
        shroudforge_loader_config::config::read_loader(root.path()).unwrap()["modules"]["runtimeDiagnostics"]
            ["enabled"],
        false
    );
    assert_eq!(status(root.path())["reason"], "snapshot-complete");
}
#[test]
fn deadline_is_independent_of_sample_interval_and_restart_works() {
    let root = tempfile::tempdir().unwrap();
    shroudforge_loader_config::config::update_loader(root.path(), |value| {
        value["modules"]["runtimeDiagnostics"]["maximumDurationSeconds"] = json!(1);
        value["modules"]["runtimeDiagnostics"]["intervalMilliseconds"] = json!(60000);
        Ok(())
    })
    .unwrap();
    request(root.path(), "start").unwrap();
    let mut session = Session::new(root.path());
    session.measure(
        "mods",
        "example",
        "on_update",
        Duration::from_millis(30),
        true,
    );
    assert_eq!(
        super::diagnostics_jobs::mod_callbacks::report(&session.measurements)["mods/example/on_update"]["slowCalls"],
        1
    );
    session.next_sample = Instant::now() + Duration::from_secs(60);
    session.started = Instant::now() - Duration::from_secs(2);
    session.tick();
    assert!(!session.active);
    assert_eq!(session.reason, "duration-complete");
    request(root.path(), "start").unwrap();
    session.next_config = Instant::now();
    session.tick();
    assert!(session.active);
    assert!(session.measurements.is_empty());
    request(root.path(), "stop").unwrap();
    session.next_config = Instant::now();
    session.tick();
    assert!(!session.active);
    assert_eq!(status(root.path())["reason"], "stopped");
}
#[test]
fn invalid_settings_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    assert!(
        shroudforge_loader_config::config::update_loader(root.path(), |value| {
            value["modules"]["runtimeDiagnostics"]["maximumDurationSeconds"] = json!(0);
            Ok(())
        })
        .is_err()
    );
    assert!(request(root.path(), "fake-action").is_err());
}
