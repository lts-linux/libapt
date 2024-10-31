//! Implementation of the InRelease file parsing.

#[cfg(not(test))] 
use log::warn;

#[cfg(test)]
use std::println as warn;

use chrono::DateTime;
use chrono::FixedOffset;
use std::collections::HashMap;

use crate::util::download;
use crate::signature::verify_in_release;
use crate::Architecture;
use crate::Distro;
use crate::{Error, ErrorType, Result};

/// Link represents a file referenced from InRelease.
/// 
/// This type is used to group all hashes for a referenced path.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub url: String,
    pub size: usize,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
}

/// The Release struct groups all data from the InRelease file.
/// 
/// When the InRelease file is parsed, all specified values from
/// [Debian Wiki InRelease specification](https://wiki.debian.org/DebianRepository/Format#A.22Release.22_files)
/// are considered.
#[derive(Debug)]
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
}

impl Release {

    /// Create a new Release struct with default values.
    fn new() -> Release {
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
        let mut release = Release::new();

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
                    let parts: Vec<String> = line
                        .split(" ")
                        .filter(|e| !e.trim().is_empty())
                        .map(|e| e.trim().to_string())
                        .collect();

                    if parts.len() != 3 {
                        warn!("Invalid file line! {line}");
                        continue;
                    }

                    let size = match parts[1].parse::<usize>() {
                        Ok(size) => size,
                        Err(e) => {
                            warn!("Invalid file size of line {line}! {e}");
                            continue;
                        }
                    };
                    let url = distro.url(&parts[2], false);

                    if !release.links.contains_key(&url) {
                        let link = Link {
                            url: url.clone(),
                            size: size,
                            md5: None,
                            sha1: None,
                            sha256: None,
                            sha512: None
                        };
                        release.links.insert(
                            url.clone(),
                            link,
                        );
                    }

                    let link = release.links.get_mut(&url).unwrap();
                    if link.size != size {
                        let link_size = link.size;
                        warn!("Size mismatch for {}! {link_size} != {size}", &url)
                    }

                    match section {
                        ReleaseSection::HashMD5 => {link.md5 =  Some(parts[0].clone())},
                        ReleaseSection::HashSHA1 => {link.sha1 =  Some(parts[0].clone())},
                        ReleaseSection::HashSHA256 => {link.sha256 =  Some(parts[0].clone())},
                        ReleaseSection::HashSHA512 => {link.sha512 =  Some(parts[0].clone())},
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
                ErrorType::InvalidReleaseFormat));
        }

        if self.architectures.is_empty() {
            return Err(Error::new(
                "No architectures provided.", 
                ErrorType::InvalidReleaseFormat));
        }

        if self.suite == None && self.codename == None {
            return Err(Error::new(
                "Neither suite nor codename provided.", 
                ErrorType::InvalidReleaseFormat));
        }

        if self.date == None {
            return Err(Error::new(
                "No date provided.", 
                ErrorType::InvalidReleaseFormat));
        }

        for key in self.links.keys() {
            let link = self.links.get(key).unwrap();
            if link.sha256 == None {
                return Err(Error::new(
                    &format!("No SHA256 hash provided for URL {key}."), 
                    ErrorType::InvalidReleaseFormat));
            }
        }

        Ok(())
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
    use crate::{Key, Distro, Release};

    #[test]
    fn parse_ubuntu_jammy_release_file() {
        // Ubuntu Jammy signing key.
        let key = Key::key("/etc/apt/trusted.gpg.d/ubuntu-keyring-2018-archive.gpg");
        
        // Ubuntu Jammy distribution.
        let distro = Distro::repo(
            "http://archive.ubuntu.com/ubuntu",
            "jammy",
            key,
        );

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
}
