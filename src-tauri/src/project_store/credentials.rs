use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::types::*;

pub(crate) const OPENROUTER_API_KEY_ENV: &str = "OPENROUTER_API_KEY";
const CREDENTIAL_STORE_FILE: &str = "credentials.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct CredentialStore {
    /// Base64-encoded API key (obfuscated, not encrypted).
    #[serde(default)]
    openrouter_api_key: Option<String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn get_execution_credential_status(app_data_dir: &Path) -> StoreResult<ExecutionCredentialStatus> {
    let has_stored_key = load_stored_openrouter_api_key(app_data_dir)?.is_some();
    Ok(build_execution_credential_status(
        has_stored_key,
        load_environment_api_key().is_some(),
    ))
}

pub fn save_execution_api_key(app_data_dir: &Path, api_key: &str) -> StoreResult<ExecutionCredentialStatus> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::message("API key cannot be empty."));
    }

    fs::create_dir_all(app_data_dir)?;
    let mut store = read_credential_store(app_data_dir);
    store.openrouter_api_key = Some(encode_key(trimmed));
    write_credential_store(app_data_dir, &store)?;

    Ok(ExecutionCredentialStatus {
        source: CredentialSource::Stored,
        has_stored_key: true,
    })
}

pub fn clear_execution_api_key(app_data_dir: &Path) -> StoreResult<ExecutionCredentialStatus> {
    let mut store = read_credential_store(app_data_dir);
    store.openrouter_api_key = None;
    if app_data_dir.join(CREDENTIAL_STORE_FILE).exists() {
        write_credential_store(app_data_dir, &store)?;
    }
    get_execution_credential_status(app_data_dir)
}

// ── Helpers (crate-visible for execution.rs) ──────────────────────────────────

pub(crate) fn load_stored_openrouter_api_key(app_data_dir: &Path) -> StoreResult<Option<String>> {
    let store = read_credential_store(app_data_dir);
    match store.openrouter_api_key {
        Some(encoded) => {
            let decoded = decode_key(&encoded);
            if decoded.trim().is_empty() { Ok(None) } else { Ok(Some(decoded)) }
        }
        None => Ok(None),
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
        CredentialSource::Stored
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

fn read_credential_store(app_data_dir: &Path) -> CredentialStore {
    let path = app_data_dir.join(CREDENTIAL_STORE_FILE);
    if !path.exists() {
        return CredentialStore::default();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => CredentialStore::default(),
    }
}

fn write_credential_store(app_data_dir: &Path, store: &CredentialStore) -> StoreResult<()> {
    let path = app_data_dir.join(CREDENTIAL_STORE_FILE);
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| ProjectStoreError::message(format!("Failed to serialize credentials: {e}")))?;
    fs::write(&path, json)?;
    Ok(())
}

fn encode_key(key: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(key.as_bytes())
}

fn decode_key(encoded: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    #[test]
    fn round_trips_api_key_through_app_data_file() {
        let dir = TempDir::new().unwrap();
        let app_data = dir.path();

        save_execution_api_key(app_data, "sk-or-v1-test123").unwrap();
        let loaded = load_stored_openrouter_api_key(app_data).unwrap();
        assert_eq!(loaded.as_deref(), Some("sk-or-v1-test123"));

        let status = get_execution_credential_status(app_data).unwrap();
        assert_eq!(status.source, CredentialSource::Stored);
        assert!(status.has_stored_key);
    }

    #[test]
    fn clear_removes_stored_key() {
        let dir = TempDir::new().unwrap();
        let app_data = dir.path();

        save_execution_api_key(app_data, "sk-or-v1-test123").unwrap();
        clear_execution_api_key(app_data).unwrap();

        let loaded = load_stored_openrouter_api_key(app_data).unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn key_is_base64_obfuscated_on_disk() {
        let dir = TempDir::new().unwrap();
        let app_data = dir.path();

        save_execution_api_key(app_data, "my-secret-key").unwrap();

        let raw = std::fs::read_to_string(app_data.join("credentials.json")).unwrap();
        assert!(!raw.contains("my-secret-key"), "plaintext key should not appear in file");
        assert!(raw.contains("bXktc2VjcmV0LWtleQ=="), "base64 encoded key should appear");
    }
}
