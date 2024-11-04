//! Implementation of the InRelease file parsing.

#[cfg(not(test))]
use log::{info, warn};

#[cfg(test)]
use std::{println as warn, println as info};

use chrono::DateTime;
use chrono::FixedOffset;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::signature::verify_in_release;
use crate::util::{download, get_etag};
use crate::Architecture;
use crate::Distro;
use crate::Link;
use crate::LinkHash;
use crate::{Error, ErrorType, Result};

/// The Release struct groups all data from the InRelease file.
///
/// When the InRelease file is parsed, all specified values from
/// [Debian Wiki InRelease specification](https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files)
/// are considered.
#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    // fields from apt release file
    hash: Option<String>,
    pub origin: Option<String>,
    pub label: Option<String>,
    pub suite: Option<String>,
    pub version: Option<String>,
    pub codename: Option<String>,
    pub date: Option<DateTime<FixedOffset>>,
    pub valid_until: Option<DateTime<FixedOffset>>,
    pub architectures: Vec<Architecture>,
    pub components: Vec<String>,
    pub description: Option<String>,
    pub links: HashMap<String, Link>,
    pub acquire_by_hash: bool,
    pub signed_by: Vec<String>,
    pub changelogs: Option<String>,
    pub snapshots: Option<String>,
    // internal data
    pub distro: Distro,
}

impl Release {
    /// Create a new Release struct with default values.
    fn new(distro: &Distro) -> Release {
        Release {
            hash: None,
            origin: None,
            label: None,
            suite: None,
            version: None,
            codename: None,
            date: None,
            valid_until: None,
            architectures: Vec::new(),
            components: Vec::new(),
            description: None,
            links: HashMap::new(),
            acquire_by_hash: false, // default is false
            signed_by: Vec::new(),
            changelogs: None,
            snapshots: None,
            distro: distro.clone(),
        }
    }

    /// Download and parse the InRelease file of the given Distro.
    pub fn from_distro(distro: &Distro) -> Result<Release> {
        // Get URL content.
        let url = distro.in_release_url()?;
        let content = download(&url)?;

        // Verify signature.
        let content = verify_in_release(content, distro)?;

        // Parse content.
        let mut section = ReleaseSection::Keywords;
        let mut release = Release::new(distro);

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with("---") {
                if line.starts_with("-----BEGIN PGP SIGNATURE") {
                    // Signature check not implemented. Stop parsing.
                    break;
                } else {
                    // Skip PGP headers.
                    continue;
                }
            } else if !line.starts_with(" ") {
                section = ReleaseSection::Keywords;
            }

            match &section {
                ReleaseSection::Keywords => {
                    if !line.contains(":") {
                        return Err(Error::new(
                            &format!("Invalid line! {line}"),
                            ErrorType::InvalidReleaseFormat,
                        ));
                    }

                    let mut parts = line.splitn(2, ":");
                    let keyword = parts.next().unwrap();
                    let value = parts.next().unwrap();

                    let keyword = keyword.to_lowercase();
                    let plain_value = value.trim();
                    let value = Some(plain_value.to_string());

                    if keyword == "hash" {
                        release.hash = value;
                    } else if keyword == "origin" {
                        release.origin = value;
                    } else if keyword == "label" {
                        release.label = value;
                    } else if keyword == "suite" {
                        release.suite = value;
                    } else if keyword == "version" {
                        release.version = value;
                    } else if keyword == "codename" {
                        release.codename = value;
                    } else if keyword == "description" {
                        release.description = value;
                    } else if keyword == "changelogs" {
                        release.changelogs = value;
                    } else if keyword == "snapshots" {
                        release.snapshots = value;
                    } else if keyword == "date" {
                        let plain_value = plain_value.replace("UTC", "+0000");
                        release.date = match DateTime::parse_from_rfc2822(&plain_value) {
                            Ok(date) => Some(date),
                            Err(e) => {
                                warn!("Parsing Release date \"{plain_value}\" failed! {e}");
                                None
                            }
                        }
                    } else if keyword == "valid-until" {
                        let plain_value = plain_value.replace("UTC", "+0000");
                        release.valid_until = match DateTime::parse_from_rfc2822(&plain_value) {
                            Ok(date) => Some(date),
                            Err(e) => {
                                warn!("Parsing Release valid until failed! {e}");
                                None
                            }
                        }
                    } else if keyword == "architectures" {
                        release.architectures = plain_value
                            .split(" ")
                            .filter_map(|e| match Architecture::from_str(e) {
                                Ok(arch) => Some(arch),
                                Err(e) => {
                                    warn!("Parsing architecture {e} failed!");
                                    None
                                }
                            })
                            .collect();
                    } else if keyword == "components" {
                        release.components = plain_value
                            .split(" ")
                            .filter(|e| !e.trim().is_empty())
                            .map(|e| e.to_string())
                            .collect();
                    } else if keyword == "acquire-by-hash" {
                        release.acquire_by_hash = match plain_value.to_lowercase().as_str() {
                            "yes" => true,
                            _ => false,
                        }
                    } else if keyword == "signed-by" {
                        release.signed_by = plain_value
                            .split(",")
                            .map(|e| e.trim().to_string())
                            .collect();
                    } else if keyword == "md5sum" {
                        section = ReleaseSection::HashMD5;
                    } else if keyword == "sha1" {
                        section = ReleaseSection::HashSHA1;
                    } else if keyword == "sha256" {
                        section = ReleaseSection::HashSHA256;
                    } else if keyword == "sha512" {
                        section = ReleaseSection::HashSHA512;
                    } else {
                        warn!("Unknown keyword: {keyword} of line {line}!");
                    }
                }
                section => {
                    let link = Link::form_release(line, distro)?;
                    let url = link.url.clone();

                    if !release.links.contains_key(&url) {
                        release.links.insert(url.clone(), link);
                    }

                    let link = release.links.get_mut(&url).unwrap();

                    match section {
                        ReleaseSection::HashMD5 => {
                            link.add_hash(line, LinkHash::Md5)?;
                        }
                        ReleaseSection::HashSHA1 => {
                            link.add_hash(line, LinkHash::Sha1)?;
                        }
                        ReleaseSection::HashSHA256 => {
                            link.add_hash(line, LinkHash::Sha256)?;
                        }
                        ReleaseSection::HashSHA512 => {
                            link.add_hash(line, LinkHash::Sha512)?;
                        }
                        _ => {}
                    };
                }
            }
        }

        Ok(release)
    }

    pub fn check_compliance(&self) -> Result<()> {
        if self.components.is_empty() {
            return Err(Error::new(
                "No components provided.",
                ErrorType::InvalidReleaseFormat,
            ));
        }

        if self.architectures.is_empty() {
            return Err(Error::new(
                "No architectures provided.",
                ErrorType::InvalidReleaseFormat,
            ));
        }

        if self.suite == None && self.codename == None {
            return Err(Error::new(
                "Neither suite nor codename provided.",
                ErrorType::InvalidReleaseFormat,
            ));
        }

        if self.date == None {
            return Err(Error::new(
                "No date provided.",
                ErrorType::InvalidReleaseFormat,
            ));
        }

        for key in self.links.keys() {
            let link = self.links.get(key).unwrap();
            if !link.hashes.contains_key(&LinkHash::Sha256) {
                return Err(Error::new(
                    &format!("No SHA256 hash provided for URL {key}."),
                    ErrorType::InvalidReleaseFormat,
                ));
            }
        }

        Ok(())
    }

    pub fn get_package_links(&self) -> Vec<(String, Architecture, Link)> {
        let mut components = Vec::new();

        for architecture in &self.architectures {
            for component in &self.components {
                let link = match self.get_package_index_link(component, architecture) {
                    Ok(link) => link,
                    Err(_) => {
                        info!("No link for component {component} and architecture {architecture}. Skipping.");
                        continue;
                    }
                };
                components.push((component.to_string(), architecture.clone(), link));
            }
        }

        components
    }

    pub fn get_package_index_link(
        &self,
        component: &str,
        architecture: &Architecture,
    ) -> Result<Link> {
        let index_url = if architecture == &Architecture::Source {
            format!("{component}/source/Sources")
        } else {
            let arch_str = architecture.to_string();
            format!("{component}/binary-{arch_str}/Packages")
        };

        let index_url = self.distro.url(&index_url, false);

        // Supported compression extensions, try form best to no compression
        let extensions = vec![".xz", ".gz", ""];

        for ext in extensions {
            // Build URL for compressed index.
            let package_index = index_url.clone() + ext;

            // Find link in release.
            // The link is mandatory to get the hash sums for verification.
            match self.links.get(&package_index) {
                Some(link) => {
                    match get_etag(&link.url) {
                        Ok(_) => return Ok(link.clone()), // Index file exists.
                        Err(_) => {
                            info!("No etag for {package_index}, trying next link.");
                            continue;
                        }
                    }
                }
                None => {
                    info!("Index {package_index} not found.");
                }
            }
        }

        // No link found.
        Err(Error::new(
            &format!("No matching package index found for component {component} and architecture {architecture}!"),
            ErrorType::DownloadFailure,
        ))
    }
}

/// Internal helper as marker for the sections of the InRelease file.
enum ReleaseSection {
    Keywords,
    HashMD5,
    HashSHA1,
    HashSHA256,
    HashSHA512,
}

#[cfg(test)]
mod tests {
    use crate::{Distro, Key, Release};

    #[test]
    fn parse_ubuntu_jammy_release_file() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        let release = Release::from_distro(&distro).unwrap();

        assert_eq!(release.origin, Some("Ubuntu".to_string()), "Origin");
        assert_eq!(release.label, Some("Ubuntu".to_string()), "Label");
        assert_eq!(release.suite, Some("jammy".to_string()), "Suite");
        assert_eq!(release.codename, Some("jammy".to_string()), "Codename");
        assert_eq!(release.version, Some("22.04".to_string()), "Version");
        assert_eq!(release.acquire_by_hash, true, "Acquire-By-Hash");

        // Parse the InRelease file.
        let release = Release::from_distro(&distro).unwrap();

        // Check for compliance with https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files.
        release.check_compliance().unwrap();
    }

    #[test]
    fn parse_ebcl_release_file() {
        // EBcL signing key.
        let key = Key::armored_key("https://linux.elektrobit.com/eb-corbos-linux/ebcl_1.0_key.pub");

        // EBcL 1.3 distribution.
        let distro = Distro::repo(
            "http://linux.elektrobit.com/eb-corbos-linux/1.3",
            "ebcl",
            key,
        );

        let release = Release::from_distro(&distro).unwrap();

        assert_eq!(release.origin, Some("Elektrobit".to_string()), "Origin");
        assert_eq!(release.suite, Some("ebcl".to_string()), "Suite");
        assert_eq!(release.codename, Some("ebcl".to_string()), "Codename");

        // Parse the InRelease file.
        let release = Release::from_distro(&distro).unwrap();

        // Check for compliance with https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files.
        release.check_compliance().unwrap();
    }

    #[test]
    fn test_wrong_key() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo(
            "http://linux.elektrobit.com/eb-corbos-linux/1.3",
            "ebcl",
            key,
        );

        match Release::from_distro(&distro) {
            Ok(_) => assert!(false), // Key verification shall fail!
            Err(_) => {}
        };
    }

    #[test]
    fn test_package_index_link() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        let release = Release::from_distro(&distro).unwrap();

        let link = release
            .get_package_index_link("main", &crate::Architecture::Amd64)
            .unwrap();
        assert_eq!(
            link.url,
            "http://archive.ubuntu.com/ubuntu/dists/jammy/main/binary-amd64/Packages.xz"
                .to_string()
        );

        match release.get_package_index_link("main", &crate::Architecture::Arm64) {
            Ok(_) => assert!(false), // Should not exist!
            Err(_) => {}             // Ok, expected.
        };
    }

    #[test]
    fn test_get_package_links() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");

        // Ubuntu Jammy distribution.
        let distro = Distro::repo("http://archive.ubuntu.com/ubuntu", "jammy", key);

        let release = Release::from_distro(&distro).unwrap();

        let components = release.get_package_links();
        println!("Components: {:?}", components);
        println!("Found {} package indices.", components.len());
        assert_eq!(components.len(), 8);
    }
}
