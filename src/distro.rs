use crate::util::join_url;
use crate::{Error, Result};

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Distro {
    url: String,
    name: Option<String>,
    path: Option<String>,
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

    pub fn url(&self, path: &str) -> String {
        join_url(&self.url, path)
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
}
