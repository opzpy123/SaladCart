use crate::error::{AppError, AppResult};

const SERVICE: &str = "com.saladcart.studio.providers";

fn entry(provider_id: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, provider_id)
        .map_err(|e| AppError::Message(format!("keychain: {e}")))
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
    entry(provider_id)?
        .get_password()
        .map_err(|e| AppError::Message(format!("keychain: {e}")))
}

pub fn has_secret(provider_id: &str) -> AppResult<bool> {
    match entry(provider_id)?.get_password() {
        Ok(value) => Ok(!value.is_empty()),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(AppError::Message(format!("keychain: {e}"))),
    }
}

pub fn delete_secret(provider_id: &str) -> AppResult<()> {
    match entry(provider_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Message(format!("keychain: {e}"))),
    }
}
