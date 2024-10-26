use crate::signature::verify_in_release;
use crate::util::download;
use crate::util::join_url;
use crate::{Error, Release, Result};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Key {
    Key(String),
    ArmoredKey(String),
    NoSignatureCheck,
}

impl Key {
    pub fn key(url: &str) -> Key {
        Key::Key(url.to_string())
    }

    pub fn armored_key(url: &str) -> Key {
        Key::ArmoredKey(url.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Distro {
    pub url: String,
    pub name: Option<String>,
    pub path: Option<String>,
    pub key: Key,
}

impl Distro {
    pub fn repo(url: &str, name: &str, key: Key) -> Distro {
        Distro {
            url: url.to_string(),
            name: Some(name.to_string()),
            path: None,
            key: key,
        }
    }

    pub fn flat_repo(url: &str, directory: &str, key: Key) -> Distro {
        Distro {
            url: url.to_string(),
            name: None,
            path: Some(directory.to_string()),
            key: key,
        }
    }

    pub fn in_release_url(&self) -> Result<String> {
        if let Some(name) = &self.name {
            let url = join_url(&self.url, "dists");
            let url = join_url(&url, &name);
            let url = join_url(&url, "InRelease");
            Ok(url)
        } else if let Some(path) = &self.path {
            let url = join_url(&self.url, &path);
            let url = join_url(&url, "InRelease");
            Ok(url)
        } else {
            Err(Error::new(
                "No distro name",
                crate::ErrorType::InvalidDistro,
            ))
        }
    }

    pub fn url(&self, path: &str, package: bool) -> String {
        if package {
            join_url(&self.url, path)
        } else {
            let path = if let Some(name) = &self.name {
                join_url(&format!("dists/{name}"), path)
            } else if let Some(dist_path) = &self.path {
                join_url(&dist_path, path)
            } else {
                assert!(false);
                path.to_string()
            };
            join_url(&self.url, &path)
        }
    }

    /// Parse the metadata of the given distro.
    pub fn parse_distro(&self) -> Result<Release> {
        let url = self.in_release_url()?;
        let content = download(&url)?;
        let content = verify_in_release(content, &self)?;
        let release = Release::parse(&content, &self)?;

        Ok(release)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distro_in_release_url() {
        let distro = Distro::flat_repo(
            "http://archive.ubuntu.com/ubuntu",
            "./",
            Key::NoSignatureCheck,
        );
        let in_release = distro.in_release_url().unwrap();
        assert!(
            in_release == "http://archive.ubuntu.com/ubuntu/InRelease",
            "InRelease url"
        );
    }

    #[test]
    fn flat_distro_in_release_url() {
        let distro = Distro::flat_repo(
            "http://archive.ubuntu.com/ubuntu",
            "path",
            Key::Key("http://archive.ubuntu.com/ubuntu/key.pub".to_string()),
        );
        let in_release = distro.in_release_url().unwrap();
        assert!(
            in_release == "http://archive.ubuntu.com/ubuntu/path/InRelease",
            "InRelease url"
        );
    }

    #[test]
    fn repo() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::NoSignatureCheck,
        );
        assert!(
            distro.url == "http://archive.ubuntu.com/ubuntu",
            "distro url"
        );
        assert!(distro.name == Some("jammy".to_string()), "distro name");
        assert!(distro.path == None, "distro path");
        assert!(distro.key == Key::NoSignatureCheck, "distro key");
    }

    #[test]
    fn repo_key() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::Key("http://archive.ubuntu.com/ubuntu/key.pub".to_string()),
        );
        assert!(
            distro.url == "http://archive.ubuntu.com/ubuntu",
            "distro url"
        );
        assert!(distro.name == Some("jammy".to_string()), "distro name");
        assert!(distro.path == None, "distro path");
        assert!(
            distro.key == Key::Key("http://archive.ubuntu.com/ubuntu/key.pub".to_string()),
            "distro key"
        );
    }

    #[test]
    fn flat_repo() {
        let distro = Distro::flat_repo(
            "http://archive.ubuntu.com/ubuntu",
            "./",
            Key::NoSignatureCheck,
        );
        assert!(
            distro.url == "http://archive.ubuntu.com/ubuntu",
            "distro url"
        );
        assert!(distro.path == Some("./".to_string()), "distro name");
        assert!(distro.name == None, "distro name");
        assert!(distro.key == Key::NoSignatureCheck, "distro key");
    }

    #[test]
    fn flat_repo_key() {
        let distro = Distro::flat_repo(
            "http://archive.ubuntu.com/ubuntu",
            "path",
            Key::Key("http://archive.ubuntu.com/ubuntu/key.pub".to_string()),
        );
        assert!(
            distro.url == "http://archive.ubuntu.com/ubuntu",
            "distro url"
        );
        assert!(distro.path == Some("path".to_string()), "distro name");
        assert!(distro.name == None, "distro name");
        assert!(
            distro.key == Key::Key("http://archive.ubuntu.com/ubuntu/key.pub".to_string()),
            "distro key"
        );
    }

    #[test]
    fn parse_ubuntu_jammy_release_file() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::NoSignatureCheck,
        );

        let mut release = distro.parse_distro().unwrap();

        assert_eq!(release.origin, Some("Ubuntu".to_string()), "Origin");
        assert_eq!(release.label, Some("Ubuntu".to_string()), "Label");
        assert_eq!(release.suite, Some("jammy".to_string()), "Suite");
        assert_eq!(release.codename, Some("jammy".to_string()), "Codename");
        assert_eq!(release.version, Some("22.04".to_string()), "Version");
        assert_eq!(release.acquire_by_hash, true, "Acquire-By-Hash");

        release.parse_components().unwrap();
    }
}
