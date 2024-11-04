//! GPG signature verification.

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

/// Get the content of an armored key as string.
///
/// If the given url starts with http a download is tried,
/// else the url is interpreted as local file path.
fn _get_key_content(url: &str) -> Result<String> {
    if url.starts_with("http") {
        info!("Download key from URL {url}.");
        match download(&url) {
            Ok(content) => Ok(content),
            Err(e) => {
                let message = format!("Download of key {url} failed! {e}");
                error!("{}", &message);
                Err(Error::new(&message, crate::ErrorType::Verification))
            }
        }
    } else {
        info!("Download key from file {url}.");
        match fs::read_to_string(url) {
            Ok(content) => Ok(content),
            Err(e) => {
                let message = format!("Reading key {url} failed! {e}");
                error!("{}", &message);
                Err(Error::new(&message, crate::ErrorType::Verification))
            }
        }
    }
}

/// Get the signing key for the given Distro.
fn _get_key(distro: &Distro) -> Result<Option<SignedPublicKey>> {
    let key = match &distro.key {
        Key::ArmoredKey(url) => {
            info!("Get armored key for {:?} from {url}.", &distro.name);
            let content = _get_key_content(&url)?;
            let (public_key, _headers_public) =
                SignedPublicKey::from_string(&content).map_err(|e| {
                    Error::new(
                        &format!("Loading key {url} failed! {e}"),
                        crate::ErrorType::Verification,
                    )
                })?;
            public_key
        }
        Key::Key(url) => {
            info!("Get de-armored key for {:?} from {url}.", &distro.name);
            let file = File::open(url).map_err(|e| {
                Error::new(
                    &format!("Loading key failed! {e}"),
                    crate::ErrorType::Verification,
                )
            })?;

            SignedPublicKey::from_bytes(BufReader::new(file)).map_err(|e| {
                info!("Signature verification skipped for {:?}.", &distro.name);
                Error::new(
                    &format!("Loading key failed! {e}"),
                    crate::ErrorType::Verification,
                )
            })?
        }
        Key::NoSignatureCheck => {
            info!("No key for distro {:?}.", &distro.name);
            return Ok(None);
        }
    };

    match key.verify() {
        Ok(_) => info!("Public key for distro {:?} is OK!", &distro.name),
        Err(e) => {
            let message = format!("Public key for distro {:?} is NOT OK! {e}", &distro.name);
            error!("{}", &message);
            return Err(Error::new(&message, crate::ErrorType::Verification));
        }
    };

    Ok(Some(key))
}

/// Verify the signature of the InRelease file.
///
/// The full content of the inline signed file is given as content.
/// The given Distro is used to specify the signing key.
pub fn verify_in_release(content: String, distro: &Distro) -> Result<String> {
    info!("Verifying signature of distro {:?}.", &distro.name);

    let key = match _get_key(distro)? {
        Some(key) => key,
        None => return Ok(content),
    };

    let (inrelease, _headers_msg) = CleartextSignedMessage::from_string(&content).unwrap();

    match inrelease.verify(&key) {
        Ok(_) => info!("InRelease signature for distro {:?} is OK!", &distro.name),
        Err(e) => {
            let message = format!(
                "InRelease signature for distro {:?} is NOT OK! {e}",
                &distro.name
            );
            error!("{}", &message);
            return Err(Error::new(&message, crate::ErrorType::Verification));
        }
    }

    Ok(inrelease.text().to_string())
}
