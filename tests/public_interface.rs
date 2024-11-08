use libapt::{Distro, Key, Release};

#[tokio::test]
async fn parse_ubuntu_jammy_release_file() {
    let distro = Distro::repo(
        "http://archive.ubuntu.com/ubuntu",
        "jammy",
        Key::NoSignatureCheck,
    );

    let release = Release::from_distro(&distro).await.unwrap();

    assert_eq!(release.origin, Some("Ubuntu".to_string()), "Origin");
    assert_eq!(release.label, Some("Ubuntu".to_string()), "Label");
    assert_eq!(release.suite, Some("jammy".to_string()), "Suite");
    assert_eq!(release.codename, Some("jammy".to_string()), "Codename");
    assert_eq!(release.version, Some("22.04".to_string()), "Version");
    assert_eq!(release.acquire_by_hash, true, "Acquire-By-Hash");
}
