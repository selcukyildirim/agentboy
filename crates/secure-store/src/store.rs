use crate::credential::StoredCredential;
use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use keyring::Entry;

const SERVICE_NAME: &str = "com.agentboy.desktop";

#[async_trait]
pub trait SecureStore: Send + Sync {
    async fn save_credential(&self, cred: &StoredCredential) -> AppResult<()>;
    async fn get_credential(&self, provider: &str) -> AppResult<Option<StoredCredential>>;
    async fn delete_credential(&self, provider: &str) -> AppResult<()>;
    async fn list_providers(&self) -> AppResult<Vec<String>>;
}

pub struct OsKeychainStore;

impl OsKeychainStore {
    pub fn new() -> Self {
        Self
    }

    fn entry_for_provider(&self, provider: &str) -> AppResult<Entry> {
        Entry::new(SERVICE_NAME, provider)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Keychain error: {}", e)))
    }
}

impl Default for OsKeychainStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecureStore for OsKeychainStore {
    async fn save_credential(&self, cred: &StoredCredential) -> AppResult<()> {
        let entry = self.entry_for_provider(&cred.provider)?;
        let payload = serde_json::to_string(cred)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Serialize error: {}", e)))?;

        entry
            .set_password(&payload)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Keychain save error: {}", e)))?;

        Ok(())
    }

    async fn get_credential(&self, provider: &str) -> AppResult<Option<StoredCredential>> {
        let entry = self.entry_for_provider(provider)?;

        match entry.get_password() {
            Ok(payload) => {
                let cred: StoredCredential = serde_json::from_str(&payload)
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("Deserialize error: {}", e)))?;
                Ok(Some(cred))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Internal(anyhow::anyhow!(
                "Keychain read error: {}",
                e
            ))),
        }
    }

    async fn delete_credential(&self, provider: &str) -> AppResult<()> {
        let entry = self.entry_for_provider(provider)?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Internal(anyhow::anyhow!(
                "Keychain delete error: {}",
                e
            ))),
        }
    }

    async fn list_providers(&self) -> AppResult<Vec<String>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::CredentialStatus;

    #[tokio::test]
    async fn test_save_and_get_credential() {
        let store = OsKeychainStore::new();
        let cred = StoredCredential {
            provider: "test_provider".to_string(),
            secret_ref: "test_secret_ref".to_string(),
            created_at: chrono::Utc::now(),
            last_validated_at: None,
            status: CredentialStatus::Active,
        };

        store.save_credential(&cred).await.unwrap();
        let retrieved = store.get_credential("test_provider").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().provider, "test_provider");

        store.delete_credential("test_provider").await.unwrap();
        let deleted = store.get_credential("test_provider").await.unwrap();
        assert!(deleted.is_none());
    }
}
