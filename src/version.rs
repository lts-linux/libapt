//! Implementation of package versions.
//!
#[cfg(not(test))]
use log::error;

#[cfg(test)]
use std::println as error;

use std::cmp::Ordering;
use std::iter::zip;

use crate::{Error, ErrorType, Result};

/// Split a Debian package version or revision into parts.
///
/// Parts are either sequences for numbers, separators or characters.
fn split_parts(version: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    if version == "" {
        return result;
    }

    let chars: Vec<char> = version.chars().collect();

    let mut digit = chars[0].is_ascii_digit();

    let mut part = String::new();
    for c in version.chars() {
        if (digit && c.is_ascii_digit()) || (!digit && !c.is_ascii_digit()) {
            part.push(c);
        } else {
            result.push(part);
            digit = !digit;

            part = String::new();
            part.push(c);
        }
    }
    result.push(part);

    result
}

/// The Version struct groups the Debian version parts.
#[derive(Eq, Debug, Clone)]
pub struct Version {
    // see https://www.debian.org/doc/debian-policy/ch-controlfields.html#version
    pub epoch: Option<u8>,
    pub version: String,
    pub revision: Option<String>,
}

impl Version {
    /// Split Debian version string into epoch, version and revision.
    fn split(version: &str) -> Result<(Option<u8>, String, Option<String>)> {
        let (epoch, version) = match version.find(':') {
            Some(pos) => {
                let epoch = &version[..pos];
                let version = &version[(pos + 1)..];

                let epoch = match epoch.parse::<u8>() {
                    Ok(epoch) => epoch,
                    Err(e) => {
                        let message = format!("Parse epoch error! {e}");
                        error!("{}", &message);
                        return Err(Error::new(&message, ErrorType::VerificationError));
                    }
                };

                (Some(epoch), version)
            }
            None => (None, version),
        };

        let (version, revision) = match version.rfind("-") {
            Some(pos) => {
                let revision = &version[(pos + 1)..];
                let version = &version[..pos];

                (version, Some(revision.to_string()))
            }
            None => (version, None),
        };

        Ok((epoch, version.to_string(), revision))
    }

    /// Parse a new Version from its string representation.
    pub fn from_str(version: &str) -> Result<Version> {
        let (epoch, version, revision) = Version::split(version)?;

        let version = Version {
            epoch: epoch,
            version: version,
            revision: revision,
        };

        Ok(version)
    }

    /// Compare two epochs.
    fn compare_epoch(&self, other: &Version) -> Ordering {
        if self.epoch == other.epoch {
            return Ordering::Equal;
        }

        // unpack numbers
        let self_epoch = match self.epoch {
            Some(epoch) => epoch,
            None => 0,
        };
        let other_epoch = match other.epoch {
            Some(epoch) => epoch,
            None => 0,
        };

        if self_epoch > other_epoch {
            Ordering::Greater
        } else if self_epoch < other_epoch {
            Ordering::Less
        } else {
            assert!(
                false,
                "This should not happen, equality is checked as very first case."
            );
            Ordering::Equal
        }
    }

    /// Compare two versions.
    fn compare_version(&self, other: &Version) -> Ordering {
        if self.version == other.version {
            return Ordering::Equal;
        }

        Version::compare_version_str(&self.version, &other.version)
    }

    /// Compare two revisions.
    fn compare_revision(&self, other: &Version) -> Ordering {
        if self.revision == other.revision {
            return Ordering::Equal;
        }

        // Add empty string as last element marker
        let self_revision = match &self.revision {
            Some(revision) => revision,
            None => "",
        };
        let other_revision = match &other.revision {
            Some(revision) => revision,
            None => "",
        };

        // algorithms for version and revision are equal
        Version::compare_version_str(self_revision, other_revision)
    }

    /// Implementation of the version and revision comparision.
    fn compare_version_str(self_version: &str, other_version: &str) -> Ordering {
        if self_version == other_version {
            return Ordering::Equal;
        }

        let mut self_parts = split_parts(self_version);
        let mut other_parts = split_parts(other_version);

        if self_parts.len() < other_parts.len() {
            self_parts.push(String::new());
        } else if self_parts.len() > other_parts.len() {
            other_parts.push(String::new());
        }

        for (s, o) in zip(self_parts, other_parts) {
            if s == o {
                continue;
            }

            if s == "" {
                return Ordering::Less;
            }
            if o == "" {
                return Ordering::Greater;
            }

            // parse numbers
            let sn = match s.parse::<u32>() {
                Ok(sn) => Some(sn),
                Err(_) => None,
            };
            let on = match o.parse::<u32>() {
                Ok(so) => Some(so),
                Err(_) => None,
            };

            if sn != None && on == None {
                return Ordering::Greater;
            } else if sn == None && on != None {
                return Ordering::Less;
            } else if sn != None && on != None {
                // compare numbers
                if let Some(sn) = sn {
                    if let Some(on) = on {
                        if sn < on {
                            return Ordering::Less;
                        } else if sn > on {
                            return Ordering::Greater;
                        }
                    }
                }
            } else {
                // compare strings
                let mut s = s.to_string();
                let mut o = o.to_string();

                let sl = s.len();
                let ol = o.len();

                let s = if sl < ol {
                    s.push('~');
                    s.chars()
                } else {
                    s.chars()
                };
                let o = if ol < sl {
                    o.push('~');
                    o.chars()
                } else {
                    o.chars()
                };

                for (sc, oc) in zip(s, o) {
                    if sc == oc {
                        continue;
                    }

                    if sc == '~' {
                        return Ordering::Less;
                    }
                    if oc == '~' {
                        return Ordering::Greater;
                    }

                    if sc < oc {
                        return Ordering::Less;
                    } else if sc > oc {
                        return Ordering::Greater;
                    }
                }
            }
        }

        assert!(
            false,
            "This should not happen, equality is checked as very first case."
        );
        Ordering::Equal
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.epoch == other.epoch
            && self.version == other.version
            && self.revision == other.revision
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        match self.compare_epoch(other) {
            Ordering::Equal => {} // comparing versions required
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
        }

        match self.compare_version(other) {
            Ordering::Equal => {} // comparing revisions required
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
        }

        match self.compare_revision(other) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_parts() {
        let version = "2.0.12";
        let parts = split_parts(version);
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], "2");
        assert_eq!(parts[1], ".");
        assert_eq!(parts[2], "0");
        assert_eq!(parts[3], ".");
        assert_eq!(parts[4], "12");

        let version = "1ubuntu1";
        let parts = split_parts(version);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "1");
        assert_eq!(parts[1], "ubuntu");
        assert_eq!(parts[2], "1");
    }

    #[test]
    fn parse_version() {
        let v = Version::from_str("2.0.12-1ubuntu1").unwrap();
        assert_eq!(v.epoch, None, "epoch");
        assert_eq!(v.version, "2.0.12", "version");
        assert_eq!(v.revision, Some("1ubuntu1".to_string()), "revision");

        let v = Version::from_str("2.0.12-1-1ubuntu1").unwrap();
        assert_eq!(v.epoch, None, "epoch");
        assert_eq!(v.version, "2.0.12-1", "version");
        assert_eq!(v.revision, Some("1ubuntu1".to_string()), "revision");

        let v = Version::from_str("1:2.0.12-1-1ubuntu1").unwrap();
        assert_eq!(v.epoch, Some(1), "epoch");
        assert_eq!(v.version, "2.0.12-1", "version");
        assert_eq!(v.revision, Some("1ubuntu1".to_string()), "revision");
    }

    #[test]
    fn compare_version() {
        let mut versions = vec![
            Version::from_str("6.8.0-39.39").unwrap(),
            Version::from_str("6.8.0-31.31").unwrap(),
            Version::from_str("6.8.0-35.31").unwrap(),
        ];

        versions.sort();

        assert_eq!(
            versions[0],
            Version::from_str("6.8.0-31.31").unwrap(),
            "6.8.0-31.31"
        );
        assert_eq!(
            versions[2],
            Version::from_str("6.8.0-39.39").unwrap(),
            "6.8.0-39.39"
        );

        let mut versions = vec![
            Version::from_str("8.0.8-0ubuntu1~24.04.1").unwrap(),
            Version::from_str("8.0.8-0ubuntu1~24.04.2").unwrap(),
            Version::from_str("8.0.7-0ubuntu1~24.04.1").unwrap(),
        ];

        versions.sort();

        assert_eq!(
            versions[0],
            Version::from_str("8.0.7-0ubuntu1~24.04.1").unwrap(),
            "8.0.7-0ubuntu1~24.04.1"
        );
        assert_eq!(
            versions[2],
            Version::from_str("8.0.8-0ubuntu1~24.04.2").unwrap(),
            "8.0.8-0ubuntu1~24.04.2"
        );

        let mut versions = vec![
            Version::from_str("2.42.10+dfsg-3ubuntu3.1").unwrap(),
            Version::from_str("2.42.10+ffsg-3ubuntu3.1").unwrap(),
            Version::from_str("2.42.10+afsg-3ubuntu3.1").unwrap(),
            Version::from_str("2.42.10+ffsg-3ubuntu3").unwrap(),
        ];

        versions.sort();

        assert_eq!(
            versions[0],
            Version::from_str("2.42.10+afsg-3ubuntu3.1").unwrap(),
            "2.42.10+afsg-3ubuntu3.1"
        );
        assert_eq!(
            versions[1],
            Version::from_str("2.42.10+dfsg-3ubuntu3.1").unwrap(),
            "2.42.10+dfsg-3ubuntu3.1"
        );
        assert_eq!(
            versions[2],
            Version::from_str("2.42.10+ffsg-3ubuntu3").unwrap(),
            "2.42.10+ffsg-3ubuntu3"
        );
        assert_eq!(
            versions[3],
            Version::from_str("2.42.10+ffsg-3ubuntu3.1").unwrap(),
            "2.42.10+ffsg-3ubuntu3.1"
        );
    }

    #[test]
    fn partial_version() {
        let vp = Version::from_str("1.66ubuntu1").unwrap();
        let vd = Version::from_str("1.66~").unwrap();
        assert!(vd < vp, "compare versions");
    }
}
