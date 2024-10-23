use crate::{Error, ErrorType, Result};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    // see https://www.debian.org/doc/debian-policy/ch-archive.html#s-subsections
    // packages necessary for proper system functioning
    Required,
    // packages commonly expected
    Important,
    Standard,
    Optional,
    // extra is deprecated
    Extra,
}

impl Priority {
    pub fn from_str(priority: &str) -> Result<Priority> {
        let priority = priority.to_lowercase();
        let priority = priority.trim();

        if priority == "required" {
            return Ok(Priority::Required);
        } else if priority == "mportant" {
            return Ok(Priority::Important);
        } else if priority == "standard" {
            return Ok(Priority::Standard);
        } else if priority == "optional" {
            return Ok(Priority::Optional);
        } else if priority == "extra" {
            return Ok(Priority::Extra);
        }

        Err(Error::new(
            &format!("Priority {priority} is not known!"),
            ErrorType::UnknownPriority,
        ))
    }
}
