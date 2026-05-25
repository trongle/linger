#[test]
fn config_loads() {
    let cfg = linger::config::Config::load().unwrap();
    assert_eq!(cfg.app.name, "Linger");
}
