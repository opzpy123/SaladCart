use crate::error::{AppError, AppResult};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const SERVICE: &str = "com.saladcart.studio.providers";
const QUERY_TIMEOUT: Duration = Duration::from_millis(300);
const READ_TIMEOUT: Duration = Duration::from_secs(2);

fn entry(provider_id: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, provider_id)
        .map_err(|e| AppError::Message(format!("keychain: {e}")))
}

fn read_password(provider_id: &str) -> Result<Option<String>, keyring::Error> {
    let entry = keyring::Entry::new(SERVICE, provider_id)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error),
    }
}

fn read_with_timeout(provider_id: &str, timeout: Duration) -> AppResult<Option<String>> {
    let (tx, rx) = mpsc::channel();
    let owned = provider_id.to_string();
    thread::spawn(move || {
        let _ = tx.send(read_password(&owned));
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(AppError::Message(format!("keychain: {error}"))),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(AppError::Message(
            "keychain read timed out; unlock the Keychain and retry".into(),
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(AppError::Message("keychain read failed".into()))
        }
    }
}

pub fn set_secret(provider_id: &str, secret: &str) -> AppResult<()> {
    if secret.trim().is_empty() {
        return delete_secret(provider_id);
    }
    entry(provider_id)?
        .set_password(secret.trim())
        .map_err(|e| AppError::Message(format!("could not store API key in the OS keychain: {e}")))
}

pub fn get_secret(provider_id: &str) -> AppResult<String> {
    read_with_timeout(provider_id, READ_TIMEOUT)?.ok_or_else(|| {
        AppError::MissingCredential(format!("no keychain entry for {provider_id}"))
    })
}

pub fn has_secret(provider_id: &str) -> AppResult<bool> {
    // A locked or unresponsive Keychain must not block startup forever.
    match read_with_timeout(provider_id, QUERY_TIMEOUT) {
        Ok(value) => Ok(value.map(|secret| !secret.is_empty()).unwrap_or(false)),
        Err(_) => Ok(false),
    }
}

pub fn delete_secret(provider_id: &str) -> AppResult<()> {
    match entry(provider_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Message(format!("keychain: {e}"))),
    }
}
