//! Implementation of package version dependencies.

use crate::{Error, ErrorType, Result, Version};

/// A VersionRelation describes the relation between two package versions.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum VersionRelation {
    StrictSmaller,
    Smaller,
    Exact,
    Larger,
    StrictLarger,
}

impl VersionRelation {
    /// Crate a new VersionRelation from it's string representation.
    pub fn from_str(relation: &str) -> Result<VersionRelation> {
        let relation = relation.to_lowercase();
        let relation = relation.trim();

        if relation == ">>" {
            return Ok(VersionRelation::StrictLarger);
        } else if relation == ">=" {
            return Ok(VersionRelation::Larger);
        } else if relation == "=" {
            return Ok(VersionRelation::Exact);
        } else if relation == "<=" {
            return Ok(VersionRelation::Smaller);
        } else if relation == "<<" {
            return Ok(VersionRelation::StrictSmaller);
        }

        Err(Error::new(
            &format!("Package version relation {relation} is not known!"),
            ErrorType::UnknownVersionRelation,
        ))
    }

    /// Test if the versions fulfill the relation.
    pub fn matches(&self, a: &Version, b: &Version) -> bool {
        match &self {
            VersionRelation::StrictSmaller => a < b,
            VersionRelation::Smaller => a <= b,
            VersionRelation::Exact => a == b,
            VersionRelation::Larger => a >= b,
            VersionRelation::StrictLarger => a > b,
        }
    }
}

/// A PackageVersion describes a package version dependency.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PackageVersion {
    pub name: String,
    pub version: Option<Version>,
    pub relation: Option<VersionRelation>,
}

impl PackageVersion {
    /// Create a PackageVersion from it's string representation.
    pub fn from_str(desc: &str) -> Result<Vec<PackageVersion>> {
        let desc = desc.trim();

        let alternatives: Vec<&str> = desc.split("|").map(|p| p.trim()).collect();

        let result: Result<Vec<PackageVersion>> = alternatives
            .iter()
            .map(|d| PackageVersion::single_form_str(d))
            .collect();

        Ok(result?)
    }

    /// Parse a single package version from string.
    fn single_form_str(desc: &str) -> Result<PackageVersion> {
        let (name, relation, version) = match desc.find(' ') {
            Some(pos) => {
                let name = &desc[..pos];
                let version = desc[pos..].trim();
                // drop brackets
                let version = &version[1..(version.len() - 1)];
                let (relation, version) = match version.find(' ') {
                    Some(pos) => {
                        let relation = version[..pos].trim();
                        let version = version[pos..].trim();

                        let relation = VersionRelation::from_str(relation)?;

                        (relation, version)
                    }
                    None => (VersionRelation::Exact, version),
                };

                let version = Version::from_str(version)?;

                (name, Some(relation), Some(version))
            }
            None => (desc, None, None),
        };

        Ok(PackageVersion {
            name: name.to_string(),
            relation: relation,
            version: version,
        })
    }

    /// Check if the given package version matches the requirement.
    pub fn matches(&self, package_version: &Version) -> bool {
        if let Some(version) = &self.version {
            let relation = match &self.relation {
                Some(relation) => relation,
                None => &VersionRelation::Exact,
            };

            println!("Self version: {:?}", version);
            println!("Other version: {:?}", package_version);
            println!("Relation: {:?}", relation);

            relation.matches(package_version, version)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_version() {
        let pv = PackageVersion::from_str("libssl3 (>= 3.0.0~~alpha1)").unwrap();
        let pv = &pv[0];
        assert_eq!(pv.name, "libssl3");
        assert_eq!(
            pv.version,
            Some(Version::from_str("3.0.0~~alpha1").unwrap())
        );
        assert_eq!(pv.relation, Some(VersionRelation::Larger));
    }

    #[test]
    fn test_version_relation() {
        let relations = vec![
            "linux-s32-headers-5.15.0-1026",
            "libc6 (>= 2.34)",
            "libelf1 (>= 0.142)",
            "libssl3 (>= 3.0.0~~alpha1)",
            "zlib1g (>= 1:1.2.3.3)",
        ];

        let pv = PackageVersion::from_str(relations[0]).unwrap();
        let pv = &pv[0];
        assert_eq!(pv.name, "linux-s32-headers-5.15.0-1026");
        assert_eq!(pv.relation, None);
        assert_eq!(pv.version, None);

        let pv = PackageVersion::from_str(relations[1]).unwrap();
        let pv = pv[0].clone();
        assert_eq!(pv.name, "libc6");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, None);
        assert_eq!(version.version, "2.34");
        assert_eq!(version.revision, None);

        let pv = PackageVersion::from_str(relations[2]).unwrap();
        let pv = pv[0].clone();
        assert_eq!(pv.name, "libelf1");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, None);
        assert_eq!(version.version, "0.142");
        assert_eq!(version.revision, None);

        let pv = PackageVersion::from_str(relations[3]).unwrap();
        let pv = pv[0].clone();
        assert_eq!(pv.name, "libssl3");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, None);
        assert_eq!(version.version, "3.0.0~~alpha1");
        assert_eq!(version.revision, None);

        let pv = PackageVersion::from_str(relations[4]).unwrap();
        let pv = pv[0].clone();
        assert_eq!(pv.name, "zlib1g");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, Some(1));
        assert_eq!(version.version, "1.2.3.3");
        assert_eq!(version.revision, None);
    }

    #[test]
    fn test_relation_matches() {
        let a = Version::from_str("1.2.3-1ubuntu5").unwrap();
        let b = Version::from_str("1.2.3-1ubuntu6").unwrap();

        let relation = VersionRelation::Exact;
        assert!(relation.matches(&a, &a));
        assert!(!relation.matches(&a, &b));
        assert!(!relation.matches(&b, &a));

        let relation = VersionRelation::Smaller;
        assert!(relation.matches(&a, &a));
        assert!(relation.matches(&a, &b));
        assert!(!relation.matches(&b, &a));

        let relation = VersionRelation::StrictSmaller;
        assert!(!relation.matches(&a, &a));
        assert!(relation.matches(&a, &b));
        assert!(!relation.matches(&b, &a));

        let relation = VersionRelation::Larger;
        assert!(relation.matches(&a, &a));
        assert!(!relation.matches(&a, &b));
        assert!(relation.matches(&b, &a));

        let relation = VersionRelation::StrictLarger;
        assert!(!relation.matches(&a, &a));
        assert!(!relation.matches(&a, &b));
        assert!(relation.matches(&b, &a));

        let version = Version::from_str("3.0.0~~alpha1").unwrap();
        let relation = VersionRelation::Larger;
        assert!(relation.matches(&version, &version))
    }

    #[test]
    fn test_package_version_matches() {
        let a = Version::from_str("3.0.0~~alpha1").unwrap();
        let b = Version::from_str("3.2.0").unwrap();
        let c = Version::from_str("3.0.0~~alpha0").unwrap();

        let pv = PackageVersion::from_str("libssl3 (>= 3.0.0~~alpha1)").unwrap();
        let pv = pv[0].clone();
        println!("PackageVersion: {:?}", pv);
        println!("Version a: {:?}", a);
        assert!(pv.matches(&a));
        assert!(pv.matches(&b));
        assert!(!pv.matches(&c));
    }

    #[test]
    fn test_package_version_alternatives() {
        let desc = "linux-s32-headers-5.15.0-1026 | libc6 (>= 2.34) | libelf1 (>= 0.142)";
        let pvs = PackageVersion::from_str(desc).unwrap();
        assert_eq!(pvs.len(), 3);
        assert_eq!(pvs[0].name, "linux-s32-headers-5.15.0-1026");
        assert_eq!(pvs[1].name, "libc6");
        assert_eq!(pvs[2].name, "libelf1");
    }
}
