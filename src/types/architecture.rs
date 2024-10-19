use crate::{Error, ErrorType, Result};

#[derive(Debug)]
pub enum Architecture {
    Amd64,
    Arm64,
    Armhf,
    I386,
    Ppc64el,
    Riscv64,
    S390x,
    All,
}

impl Architecture {
    pub fn from_str(arch: &str) -> Result<Architecture> {
        let arch = arch.to_lowercase();
        let arch = arch.trim();

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
        }

        Err(Error::new(
            &format!("Architecture {arch} is not known!"),
            ErrorType::UnknownArchitecture,
        ))
    }
}
