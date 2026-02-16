use std::fs;
use std::io;
use std::path::PathBuf;

use super::verify::ValidationCache;

const LICENSE_DIR: &str = "claude-status";
const LICENSE_FILE: &str = "license.key";
const CACHE_FILE: &str = "license-cache.json";

pub struct LicenseStorage {
    base_dir: PathBuf,
}

impl LicenseStorage {
    pub fn new() -> Self {
        let base_dir = Self::default_dir();
        Self { base_dir }
    }

    #[cfg(test)]
    pub fn with_dir(dir: PathBuf) -> Self {
        Self { base_dir: dir }
    }

    fn default_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join(LICENSE_DIR)
    }

    fn ensure_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)
    }

    fn key_path(&self) -> PathBuf {
        self.base_dir.join(LICENSE_FILE)
    }

    fn cache_path(&self) -> PathBuf {
        self.base_dir.join(CACHE_FILE)
    }

    /// Load the stored license key, if any.
    pub fn load_key(&self) -> Option<String> {
        fs::read_to_string(self.key_path())
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Save a license key to disk.
    pub fn save_key(&self, key: &str) -> io::Result<()> {
        self.ensure_dir()?;
        let path = self.key_path();
        fs::write(&path, key.trim())?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Remove the stored license key.
    pub fn remove_key(&self) -> io::Result<()> {
        let path = self.key_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Load the cached validation result.
    pub fn load_cache(&self) -> Option<ValidationCache> {
        let data = fs::read_to_string(self.cache_path()).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Save a validation cache to disk.
    pub fn save_cache(&self, cache: &ValidationCache) -> io::Result<()> {
        self.ensure_dir()?;
        let json =
            serde_json::to_string_pretty(cache).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(self.cache_path(), json)
    }

    /// Remove the cached validation.
    pub fn remove_cache(&self) {
        let _ = fs::remove_file(self.cache_path());
    }
}

impl Default for LicenseStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::license::LicenseTier;

    #[test]
    fn test_save_and_load_key() {
        let dir = std::env::temp_dir().join(format!("claude-status-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let storage = LicenseStorage::with_dir(dir.clone());

        let key = "CS-PRO-A3F2-9D8E-C4B1-7F0A";
        storage.save_key(key).unwrap();
        let loaded = storage.load_key().unwrap();
        assert_eq!(loaded, key);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_key_missing() {
        let dir = std::env::temp_dir().join(format!("claude-status-test-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let storage = LicenseStorage::with_dir(dir.clone());

        assert!(storage.load_key().is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_key() {
        let dir = std::env::temp_dir().join(format!("claude-status-test-rm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let storage = LicenseStorage::with_dir(dir.clone());

        storage.save_key("CS-PRO-AAAA-BBBB-CCCC-DDDD").unwrap();
        assert!(storage.load_key().is_some());

        storage.remove_key().unwrap();
        assert!(storage.load_key().is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_and_load_cache() {
        let dir = std::env::temp_dir().join(format!("claude-status-test-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let storage = LicenseStorage::with_dir(dir.clone());

        let cache = ValidationCache {
            valid: true,
            tier: LicenseTier::Pro,
            expires: None,
            features: vec!["cost_tracking".to_string()],
            validated_at: Utc::now(),
        };
        storage.save_cache(&cache).unwrap();

        let loaded = storage.load_cache().unwrap();
        assert!(loaded.valid);
        assert_eq!(loaded.tier, LicenseTier::Pro);
        assert_eq!(loaded.features.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_remove_cache() {
        let dir = std::env::temp_dir().join(format!("claude-status-test-rm-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let storage = LicenseStorage::with_dir(dir.clone());

        let cache = ValidationCache {
            valid: true,
            tier: LicenseTier::Pro,
            expires: None,
            features: vec![],
            validated_at: Utc::now(),
        };
        storage.save_cache(&cache).unwrap();
        assert!(storage.load_cache().is_some());

        storage.remove_cache();
        assert!(storage.load_cache().is_none());

        let _ = fs::remove_dir_all(&dir);
    }
}
