use crate::{Error, ErrorType, Result, Version};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct PackageVersion {
    pub name: String,
    pub version: Option<Version>,
    pub relation: Option<VersionRelation>,
}

impl PackageVersion {
    pub fn from_str(desc: &str) -> Result<PackageVersion> {
        let desc = desc.trim();

        let (name, relation, version) = match desc.find(' ') {
            Some(pos) => {
                let name = &desc[..pos];
                let version = desc[pos..].trim();
                // drop brackets
                let version = &version[1..(version.len()-1)];
                let (relation, version) = match version.find(' ') {
                    Some(pos) => {
                        let relation = version[..pos].trim();
                        let version = version[pos..].trim();

                        let relation = VersionRelation::from_str(relation)?;

                        (relation, version)
                    },
                    None => (VersionRelation::Exact, version),
                };

                let version = Version::from_str(version)?;

                (name, Some(relation), Some(version))

            },
            None => (desc, None, None),
        };


        Ok(PackageVersion {
            name: name.to_string(),
            relation: relation,
            version: version,
        })
    }
}
