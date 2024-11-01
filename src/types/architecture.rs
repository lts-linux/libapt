use crate::Result;
use std::fmt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Architecture {
    Amd64,
    Arm64,
    Armhf,
    I386,
    Ppc64el,
    Riscv64,
    S390x,
    All,
    Source,
    Any,
    X32,
    Other(String),
}

impl Architecture {
    pub fn from_str(arch: &str) -> Result<Architecture> {
        let arch = arch.to_lowercase();
        let arch = if arch.starts_with("linux-") {
            &arch[6..].trim()
        } else {
            arch.trim()
        };

        if arch == "amd64" {
            return Ok(Architecture::Amd64);
        } else if arch == "arm64" {
            return Ok(Architecture::Arm64);
        } else if arch == "armhf" {
            return Ok(Architecture::Armhf);
        } else if arch == "i386" {
            return Ok(Architecture::I386);
        } else if arch == "ppc64el" {
            return Ok(Architecture::Ppc64el);
        } else if arch == "riscv64" {
            return Ok(Architecture::Riscv64);
        } else if arch == "s390x" {
            return Ok(Architecture::S390x);
        } else if arch == "all" {
            return Ok(Architecture::All);
        } else if arch == "source" {
            return Ok(Architecture::Source);
        } else if arch == "any" {
            return Ok(Architecture::Any);
        } else if arch == "x32" {
            return Ok(Architecture::X32);
        }

        Ok(Architecture::Other(arch.to_string()))
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Architecture::All => "all",
            Architecture::Amd64 => "amd64",
            Architecture::Arm64 => "arm64",
            Architecture::Armhf => "armhf",
            Architecture::I386 => "i386",
            Architecture::Ppc64el => "ppc64el",
            Architecture::Riscv64 => "riscv64",
            Architecture::S390x => "s390x",
            Architecture::Source => "source",
            Architecture::Any => "any",
            Architecture::X32 => "x32",
            Architecture::Other(s) => &format!("Other({s})"),
        };

        write!(f, "{}", name)
    }
}
