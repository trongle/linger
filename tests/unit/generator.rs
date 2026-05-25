#[test]
fn generator_dry_run() {
    let args = linger::generator::DomainArgs {
        name: "Post".into(),
        fields: vec![],
        crud: false,
        no_web: false,
        dry_run: true,
        yes: true,
    };
    let files = linger::generator::generate_domain(&args).unwrap();
    assert!(files.iter().any(|p| p.ends_with("src/posts/mod.rs")));
}
