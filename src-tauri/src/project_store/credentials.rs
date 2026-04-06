use std::env;

use crate::types::*;

pub(crate) const OPENROUTER_API_KEY_ENV: &str = "OPENROUTER_API_KEY";
const OPENROUTER_KEYCHAIN_SERVICE: &str = "com.blossomz37.diamondrunner";
const OPENROUTER_KEYCHAIN_ACCOUNT: &str = "openrouter-api-key";

// ── Public API ────────────────────────────────────────────────────────────────

pub fn get_execution_credential_status() -> StoreResult<ExecutionCredentialStatus> {
    // Keychain access can fail in unsigned dev builds or sandboxed contexts.
    // Treat errors as "no stored key" so the app can still load.
    let has_stored_key = match load_stored_openrouter_api_key() {
        Ok(key) => key.is_some(),
        Err(error) => {
            eprintln!("[diamond] keychain probe failed (non-fatal): {error}");
            false
        }
    };
    Ok(build_execution_credential_status(
        has_stored_key,
        load_environment_api_key().is_some(),
    ))
}

pub fn save_execution_api_key(api_key: &str) -> StoreResult<ExecutionCredentialStatus> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::message("API key cannot be empty."));
    }

    openrouter_keyring_entry()?
        .set_password(trimmed)
        .map_err(keyring_error)?;

    // Return a definitive success status after a confirmed write.
    // Re-probing via get_execution_credential_status() can silently fail in unsigned
    // dev builds — get_password() returns an error that is swallowed as "no key", which
    // makes the frontend revert to the empty input state even though the write succeeded.
    Ok(ExecutionCredentialStatus {
        source: CredentialSource::Keychain,
        has_stored_key: true,
    })
}

pub fn clear_execution_api_key() -> StoreResult<ExecutionCredentialStatus> {
    match openrouter_keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => get_execution_credential_status(),
        Err(error) => Err(keyring_error(error)),
    }
}

// ── Helpers (crate-visible for execution.rs) ──────────────────────────────────

pub(crate) fn load_stored_openrouter_api_key() -> StoreResult<Option<String>> {
    match openrouter_keyring_entry()?.get_password() {
        Ok(password) if password.trim().is_empty() => Ok(None),
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(keyring_error(error)),
    }
}

pub(crate) fn load_environment_api_key() -> Option<String> {
    match env::var(OPENROUTER_API_KEY_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

pub(crate) fn select_openrouter_api_key(
    stored_key: Option<String>,
    environment_key: Option<String>,
) -> Option<String> {
    stored_key
        .filter(|value| !value.trim().is_empty())
        .or_else(|| environment_key.filter(|value| !value.trim().is_empty()))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_execution_credential_status(
    has_stored_key: bool,
    has_environment_key: bool,
) -> ExecutionCredentialStatus {
    let source = if has_stored_key {
        CredentialSource::Keychain
    } else if has_environment_key {
        CredentialSource::Environment
    } else {
        CredentialSource::Missing
    };

    ExecutionCredentialStatus {
        source,
        has_stored_key,
    }
}

fn openrouter_keyring_entry() -> StoreResult<keyring::Entry> {
    keyring::Entry::new(OPENROUTER_KEYCHAIN_SERVICE, OPENROUTER_KEYCHAIN_ACCOUNT)
        .map_err(keyring_error)
}

fn keyring_error(error: keyring::Error) -> ProjectStoreError {
    ProjectStoreError::message(format!("Credential storage failed: {error}"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_stored_execution_api_key_over_environment() {
        let selected = select_openrouter_api_key(
            Some("stored-key".to_string()),
            Some("env-key".to_string()),
        );

        assert_eq!(selected.as_deref(), Some("stored-key"));
    }

    #[test]
    fn falls_back_to_environment_execution_api_key() {
        let selected = select_openrouter_api_key(None, Some("env-key".to_string()));

        assert_eq!(selected.as_deref(), Some("env-key"));
    }

    #[test]
    fn reports_missing_execution_credentials_when_nothing_is_available() {
        let status = build_execution_credential_status(false, false);

        assert_eq!(status.source, CredentialSource::Missing);
        assert!(!status.has_stored_key);
    }
}
