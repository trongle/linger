#[test]
fn templates_render() {
    let cfg = linger::config::Config::load().unwrap();
    let t = linger::templates::Templates::load(&cfg.templates).unwrap();
    let html = t
        .render(
            "pages/home/index.html",
            serde_json::json!({"app":{"name":"Linger"}}),
        )
        .unwrap();
    assert!(html.contains("Linger"));
}
