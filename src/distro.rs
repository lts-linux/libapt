//! Struct Distro and related structs and enums.

use crate::util::join_url;
use crate::{Error, Result};

/// The enum Key is used to wrap the apt repository verification key.
///
/// A non-armored key is a _Key::Key_.
/// Non-armored keys are used by apt and stored in `/etc/apt/trusted.gpg.d/`.
///
/// A armored key is a _Key::ArmoredKey_.
/// Armored keys are typically used if a key is provided for download.
///
/// The type _Key::NoSignatureCheck_ can be used to skip the verification of
/// the distribution index.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Key {
    Key(String),
    ArmoredKey(String),
    NoSignatureCheck,
}

impl Key {
    /// Wrap a new non-armored key location.
    pub fn key(url: &str) -> Key {
        Key::Key(url.to_string())
    }

    /// Wrap a new armored key location.
    pub fn armored_key(url: &str) -> Key {
        Key::ArmoredKey(url.to_string())
    }
}

/// The Distro groups all information required to locate the
/// distribution main index _InRelease_ file.
///
/// The struct can handle flat and default repositories.
/// In case of a flat repository the _name_ is _None_ and the _path_ is used.
/// In case of a default repository the _path_ is _None_ and the _name_ is used.
///
/// If _name_ and _path_ are none, the struct is invalid.
///
/// If a flat repo which makes not use of a subfolder, e.g. Suse Open Build Service,
/// the path _./_ can be used, like in apt source lists.
#[derive(Debug, Clone)]
pub struct Distro {
    pub url: String,
    pub name: Option<String>,
    pub path: Option<String>,
    pub key: Key,
}

impl Distro {
    /// Create a new default repo location description.
    pub fn repo(url: &str, name: &str, key: Key) -> Distro {
        Distro {
            url: url.to_string(),
            name: Some(name.to_string()),
            path: None,
            key: key,
        }
    }

    /// Create a new flat repo location description.
    pub fn flat_repo(url: &str, directory: &str, key: Key) -> Distro {
        Distro {
            url: url.to_string(),
            name: None,
            path: Some(directory.to_string()),
            key: key,
        }
    }

    /// Get the URL of the _InRelease_ index file.
    ///
    /// Returns an error if _name_ and _path_ are _None_.
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

    /// Create a URL using the given path.
    ///
    /// This method can be used to get valid index urls when the _package_ flag is false.
    /// In this case, the location of the _InRelease_ file is used as a base.
    ///
    /// This method can be used to get valid package urls when the _package_ flag is true.
    /// In this case, the root location is used as a base.
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
        let key = Key::key("http://archive.ubuntu.com/ubuntu/key.gpg");

        let distro = Distro::flat_repo("http://archive.ubuntu.com/ubuntu", "path", key);

        let in_release = distro.in_release_url().unwrap();

        assert_eq!(
            in_release,
            "http://archive.ubuntu.com/ubuntu/path/InRelease"
        );
    }

    #[test]
    fn repo() {
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            Key::NoSignatureCheck,
        );

        assert_eq!(distro.url, "http://archive.ubuntu.com/ubuntu");
        assert_eq!(distro.name, Some("jammy".to_string()));
        assert_eq!(distro.path, None, "distro path");
        assert_eq!(distro.key, Key::NoSignatureCheck);
    }

    #[test]
    fn repo_key() {
        let key = Key::armored_key("http://archive.ubuntu.com/ubuntu/key.pub");

        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        assert_eq!(distro.url, "http://archive.ubuntu.com/ubuntu");
        assert_eq!(distro.name, Some("jammy".to_string()));
        assert_eq!(distro.path, None);
        assert_eq!(
            distro.key,
            Key::ArmoredKey("http://archive.ubuntu.com/ubuntu/key.pub".to_string())
        );
    }

    #[test]
    fn flat_repo() {
        let distro = Distro::flat_repo(
            "http://archive.ubuntu.com/ubuntu",
            "./",
            Key::NoSignatureCheck,
        );

        assert_eq!(distro.url, "http://archive.ubuntu.com/ubuntu");
        assert_eq!(distro.path, Some("./".to_string()));
        assert_eq!(distro.name, None);
        assert_eq!(distro.key, Key::NoSignatureCheck);
    }

    #[test]
    fn flat_repo_key() {
        let key = Key::armored_key("http://archive.ubuntu.com/ubuntu/key.pub");

        let distro = Distro::flat_repo("http://archive.ubuntu.com/ubuntu", "path", key);

        assert_eq!(distro.url, "http://archive.ubuntu.com/ubuntu");
        assert_eq!(distro.path, Some("path".to_string()));
        assert_eq!(distro.name, None);
        assert_eq!(
            distro.key,
            Key::ArmoredKey("http://archive.ubuntu.com/ubuntu/key.pub".to_string())
        );
    }
}
