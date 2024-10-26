use crate::{Error, ErrorType, Result, Version};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VersionRelation {
    StrictSmaller,
    Smaller,
    Exact,
    Larger,
    StrictLarger,
}

impl VersionRelation {
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
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageVersion {
    pub name: String,
    pub version: Option<Version>,
    pub relation: Option<VersionRelation>,
}

impl PackageVersion {
    pub fn from_str(desc: &str) -> Result<PackageVersion> {
        let desc = desc.trim();

        // TODO: fix
        let desc = match desc.find("|") {
            Some(pos) => {
                log::error!("Alternative dependencies are not implemented!");
                desc[..pos].trim()
            },
            None => desc,
        };

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(pv.name, "linux-s32-headers-5.15.0-1026");
        assert_eq!(pv.relation, None);
        assert_eq!(pv.version, None);

        let pv = PackageVersion::from_str(relations[1]).unwrap();
        assert_eq!(pv.name, "libc6");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, None);
        assert_eq!(version.version, "2.34");
        assert_eq!(version.revision, None);

        let pv = PackageVersion::from_str(relations[2]).unwrap();
        assert_eq!(pv.name, "libelf1");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, None);
        assert_eq!(version.version, "0.142");
        assert_eq!(version.revision, None);

        let pv = PackageVersion::from_str(relations[3]).unwrap();
        assert_eq!(pv.name, "libssl3");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, None);
        assert_eq!(version.version, "3.0.0~~alpha1");
        assert_eq!(version.revision, None);

        let pv = PackageVersion::from_str(relations[4]).unwrap();
        assert_eq!(pv.name, "zlib1g");
        assert_eq!(pv.relation.unwrap(), VersionRelation::Larger);
        let version = pv.version.unwrap();
        assert_eq!(version.epoch, Some(1));
        assert_eq!(version.version, "1.2.3.3");
        assert_eq!(version.revision, None);
    }
}
