use crate::error::{AppError, Result};
use uuid::Uuid;

const SERVICE: &str = "Dictatr";

fn key_for(provider_id: Uuid) -> String {
    format!("provider-{}", provider_id)
}

pub fn set_api_key(provider_id: Uuid, key: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &key_for(provider_id))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.set_password(key).map_err(|e| AppError::Keyring(e.to_string()))
}

pub fn get_api_key(provider_id: Uuid) -> Result<String> {
    let entry = keyring::Entry::new(SERVICE, &key_for(provider_id))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.get_password().map_err(|e| AppError::Keyring(e.to_string()))
}

pub fn delete_api_key(provider_id: Uuid) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &key_for(provider_id))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.delete_credential().map_err(|e| AppError::Keyring(e.to_string()))
}

/// Generic named-secret API for non-provider secrets (e.g. Cloudflare Access
/// service-token secrets). Key format: `named-{name}`.
pub fn set_named_secret(name: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &format!("named-{name}"))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.set_password(value).map_err(|e| AppError::Keyring(e.to_string()))
}

pub fn get_named_secret(name: &str) -> Result<String> {
    let entry = keyring::Entry::new(SERVICE, &format!("named-{name}"))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.get_password().map_err(|e| AppError::Keyring(e.to_string()))
}

pub fn delete_named_secret(name: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &format!("named-{name}"))
        .map_err(|e| AppError::Keyring(e.to_string()))?;
    entry.delete_credential().map_err(|e| AppError::Keyring(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires real OS keyring, run only manually"]
    fn roundtrip_key() {
        let id = Uuid::new_v4();
        set_api_key(id, "secret-xyz").unwrap();
        assert_eq!(get_api_key(id).unwrap(), "secret-xyz");
        delete_api_key(id).unwrap();
    }
}
