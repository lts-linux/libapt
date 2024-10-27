#[cfg(not(test))] 
use log::{error, info};

#[cfg(test)]
use std::{println as error, println as info};

use pgp::cleartext::CleartextSignedMessage;
use pgp::{Deserializable, SignedPublicKey};
use std::fs::{self, File};
use std::io::BufReader;

use crate::util::download;
use crate::{Distro, Error, Key, Result};

fn _get_key_content(url: &str) -> Result<String> {
    if url.starts_with("http") {
        match download(&url) {
            Ok(content) => Ok(content),
            Err(e) => {
                let message = format!("Download of key {url} failed! {e}");
                error!("{}", &message);
                Err(Error::new(&message, crate::ErrorType::VerificationError))
            }
        }
    } else {
        match fs::read_to_string(url) {
            Ok(content) => Ok(content),
            Err(e) => {
                let message = format!("Reading key {url} failed! {e}");
                error!("{}", &message);
                Err(Error::new(&message, crate::ErrorType::VerificationError))
            }
        }
    }
}

fn _get_key(distro: &Distro) -> Result<Option<SignedPublicKey>> {
    let key = match &distro.key {
        Key::ArmoredKey(url) => {
            let content = _get_key_content(&url)?;
            let (public_key, _headers_public) =
                SignedPublicKey::from_string(&content).map_err(|e| {
                    Error::new(
                        &format!("Loading key {url} failed! {e}"),
                        crate::ErrorType::VerificationError,
                    )
                })?;
            public_key
        }
        Key::Key(url) => {
            let file = File::open(url).map_err(|e| {
                Error::new(
                    &format!("Loading key failed! {e}"),
                    crate::ErrorType::VerificationError,
                )
            })?;

            SignedPublicKey::from_bytes(BufReader::new(file)).map_err(|e| {
                Error::new(
                    &format!("Loading key failed! {e}"),
                    crate::ErrorType::VerificationError,
                )
            })?
        }
        Key::NoSignatureCheck => {
            info!("No key for distro {:?}.", distro);
            return Ok(None);
        }
    };

    match key.verify() {
        Ok(_) => info!("Public key for distro {:?} is OK!", distro),
        Err(e) => {
            let message = format!("Public key for distro {:?} is NOT OK! {e}", distro);
            error!("{}", &message);
            return Err(Error::new(&message, crate::ErrorType::VerificationError));
        }
    };

    Ok(Some(key))
}

pub fn verify_in_release(content: String, distro: &Distro) -> Result<String> {
    let key = match _get_key(distro)? {
        Some(key) => key,
        None => return Ok(content),
    };

    let (inrelease, _headers_msg) = CleartextSignedMessage::from_string(&content).unwrap();

    match inrelease.verify(&key) {
        Ok(_) => info!("InRelease signature for distro {:?} is OK!", distro),
        Err(e) => {
            let message = format!("InRelease signature for distro {:?} is NOT OK! {e}", distro);
            error!("{}", &message);
            return Err(Error::new(&message, crate::ErrorType::VerificationError));
        }
    }

    Ok(inrelease.text().to_string())
}
