use mote::config::Manifest;

#[test]
fn downloadable_release_manifest_matches_the_public_schema() {
    let manifest = Manifest::from_yaml(include_str!("../examples/release-agent.yaml"))
        .expect("the shipped agent.yaml must parse without manual field corrections");
    assert_eq!(manifest.capabilities, vec!["list_dir", "read_file"]);
    assert_eq!(manifest.models[0].auth_env.as_deref(), Some("MOTE_API_KEY"));
    assert!(!manifest.unsafe_shell_enabled());
}
