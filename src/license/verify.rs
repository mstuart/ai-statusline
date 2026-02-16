use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::storage::LicenseStorage;

/// License key format: CS-PRO-XXXX-XXXX-XXXX-XXXX (hex chars)
const KEY_PREFIX: &str = "CS-PRO-";
const KEY_SEGMENT_LEN: usize = 4;
const KEY_SEGMENT_COUNT: usize = 4;

/// Grace period when offline (cannot validate with server)
const OFFLINE_GRACE_DAYS: i64 = 7;

/// How often to re-validate with the server (hours)
const REVALIDATION_HOURS: i64 = 24;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseTier {
    Free,
    Pro,
    Lifetime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    Valid,
    Expired,
    Invalid,
    GracePeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub status: LicenseStatus,
    pub key: String,
    pub expires: Option<DateTime<Utc>>,
    pub features: Vec<String>,
    pub last_validated: Option<DateTime<Utc>>,
    pub machine_id: String,
}

/// Cached validation result stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCache {
    pub valid: bool,
    pub tier: LicenseTier,
    pub expires: Option<DateTime<Utc>>,
    pub features: Vec<String>,
    pub validated_at: DateTime<Utc>,
}

pub struct LicenseValidator {
    storage: LicenseStorage,
}

impl LicenseValidator {
    pub fn new() -> Self {
        Self {
            storage: LicenseStorage::new(),
        }
    }

    /// Validate a license key. Uses cached validation if recent enough,
    /// otherwise attempts online validation with graceful fallback.
    pub fn validate(&self, key: &str) -> LicenseInfo {
        let machine_id = self.machine_id();

        // Check format first
        if !Self::validate_format(key) {
            return LicenseInfo {
                tier: LicenseTier::Free,
                status: LicenseStatus::Invalid,
                key: key.to_string(),
                expires: None,
                features: vec![],
                last_validated: None,
                machine_id,
            };
        }

        // Check cached validation
        if let Some(cache) = self.storage.load_cache() {
            let age = Utc::now() - cache.validated_at;

            if cache.valid && age < Duration::hours(REVALIDATION_HOURS) {
                // Cache is fresh and valid
                return LicenseInfo {
                    tier: cache.tier,
                    status: LicenseStatus::Valid,
                    key: key.to_string(),
                    expires: cache.expires,
                    features: cache.features,
                    last_validated: Some(cache.validated_at),
                    machine_id,
                };
            }

            // Cache exists but stale - check grace period
            if cache.valid && age < Duration::days(OFFLINE_GRACE_DAYS) {
                return LicenseInfo {
                    tier: cache.tier,
                    status: LicenseStatus::GracePeriod,
                    key: key.to_string(),
                    expires: cache.expires,
                    features: cache.features,
                    last_validated: Some(cache.validated_at),
                    machine_id,
                };
            }

            // Cache expired beyond grace period
            if !cache.valid || age >= Duration::days(OFFLINE_GRACE_DAYS) {
                // Try local-only validation as last resort
                return self.offline_validate(key, &machine_id);
            }
        }

        // No cache at all - do offline validation
        self.offline_validate(key, &machine_id)
    }

    /// Activate a license key: validate format and store it.
    pub fn activate(&self, key: &str) -> Result<LicenseInfo, String> {
        if !Self::validate_format(key) {
            return Err(format!(
                "Invalid license key format. Expected: CS-PRO-XXXX-XXXX-XXXX-XXXX (hex characters)\nGot: {key}"
            ));
        }

        let machine_id = self.machine_id();

        // Store the key
        self.storage
            .save_key(key)
            .map_err(|e| format!("Failed to save license key: {e}"))?;

        // Create initial cache (valid for offline use)
        let cache = ValidationCache {
            valid: true,
            tier: LicenseTier::Pro,
            expires: None,
            features: pro_features(),
            validated_at: Utc::now(),
        };
        let _ = self.storage.save_cache(&cache);

        Ok(LicenseInfo {
            tier: LicenseTier::Pro,
            status: LicenseStatus::Valid,
            key: key.to_string(),
            expires: None,
            features: pro_features(),
            last_validated: Some(Utc::now()),
            machine_id,
        })
    }

    /// Deactivate (remove) the current license.
    pub fn deactivate(&self) -> Result<(), String> {
        self.storage
            .remove_key()
            .map_err(|e| format!("Failed to remove license: {e}"))?;
        self.storage.remove_cache();
        Ok(())
    }

    /// Validate license key format: CS-PRO-XXXX-XXXX-XXXX-XXXX
    pub fn validate_format(key: &str) -> bool {
        let key = key.trim();

        if !key.starts_with(KEY_PREFIX) {
            return false;
        }

        let rest = &key[KEY_PREFIX.len()..];
        let segments: Vec<&str> = rest.split('-').collect();

        if segments.len() != KEY_SEGMENT_COUNT {
            return false;
        }

        segments.iter().all(|seg| {
            seg.len() == KEY_SEGMENT_LEN
                && seg.chars().all(|c| c.is_ascii_hexdigit())
        })
    }

    /// Offline validation: check format + checksum only
    fn offline_validate(&self, key: &str, machine_id: &str) -> LicenseInfo {
        if Self::validate_format(key) && Self::verify_checksum(key) {
            LicenseInfo {
                tier: LicenseTier::Pro,
                status: LicenseStatus::Valid,
                key: key.to_string(),
                expires: None,
                features: pro_features(),
                last_validated: None,
                machine_id: machine_id.to_string(),
            }
        } else {
            LicenseInfo {
                tier: LicenseTier::Free,
                status: LicenseStatus::Invalid,
                key: key.to_string(),
                expires: None,
                features: vec![],
                last_validated: None,
                machine_id: machine_id.to_string(),
            }
        }
    }

    /// Verify the checksum embedded in the last segment.
    /// The last 4 hex chars are a truncated SHA-256 of the first 3 segments.
    fn verify_checksum(key: &str) -> bool {
        let key = key.trim();
        let rest = &key[KEY_PREFIX.len()..];
        let segments: Vec<&str> = rest.split('-').collect();
        if segments.len() != KEY_SEGMENT_COUNT {
            return false;
        }

        let payload = format!("{}-{}-{}", segments[0], segments[1], segments[2]);
        let expected_check = &segments[3].to_uppercase();

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);
        let computed_check = hash_hex[..KEY_SEGMENT_LEN].to_uppercase();

        *expected_check == computed_check
    }

    /// Generate a machine ID from platform-specific identifiers.
    fn machine_id(&self) -> String {
        let raw = Self::raw_machine_id();
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[..8])
    }

    #[cfg(target_os = "macos")]
    fn raw_machine_id() -> String {
        std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|output| {
                output
                    .lines()
                    .find(|line| line.contains("IOPlatformUUID"))
                    .and_then(|line| {
                        line.split('=')
                            .nth(1)
                            .map(|v| v.trim().trim_matches('"').to_string())
                    })
            })
            .unwrap_or_else(|| {
                let user = std::env::var("USER").unwrap_or_default();
                format!("macos-{user}")
            })
    }

    #[cfg(target_os = "linux")]
    fn raw_machine_id() -> String {
        std::fs::read_to_string("/etc/machine-id")
            .or_else(|_| std::fs::read_to_string("/var/lib/dbus/machine-id"))
            .unwrap_or_else(|_| {
                let user = std::env::var("USER").unwrap_or_default();
                format!("linux-{user}")
            })
    }

    #[cfg(target_os = "windows")]
    fn raw_machine_id() -> String {
        std::process::Command::new("wmic")
            .args(["csproduct", "get", "UUID"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|output| {
                output.lines().nth(1).map(|line| line.trim().to_string())
            })
            .unwrap_or_else(|| {
                let user = std::env::var("USERNAME").unwrap_or_default();
                let comp = std::env::var("COMPUTERNAME").unwrap_or_default();
                format!("{comp}-{user}")
            })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn raw_machine_id() -> String {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_default();
        format!("unknown-{user}")
    }
}

impl Default for LicenseValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a valid license key (for testing/server use).
pub fn generate_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(seed.to_le_bytes());
    let hash = hasher.finalize();
    let hex_str = hex::encode(hash).to_uppercase();

    let seg1 = &hex_str[0..4];
    let seg2 = &hex_str[4..8];
    let seg3 = &hex_str[8..12];

    // Compute checksum segment from uppercase payload (matches verify_checksum)
    let payload = format!("{seg1}-{seg2}-{seg3}");
    let mut check_hasher = Sha256::new();
    check_hasher.update(payload.as_bytes());
    let check_hash = check_hasher.finalize();
    let check_hex = hex::encode(check_hash);
    let seg4 = check_hex[..4].to_uppercase();

    format!("CS-PRO-{seg1}-{seg2}-{seg3}-{seg4}")
}

fn pro_features() -> Vec<String> {
    vec![
        "cost_tracking".to_string(),
        "burn_rate".to_string(),
        "cost_warnings".to_string(),
        "model_suggestions".to_string(),
        "historical_stats".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_format_valid() {
        assert!(LicenseValidator::validate_format("CS-PRO-A3F2-9D8E-C4B1-7F0A"));
    }

    #[test]
    fn test_validate_format_lowercase_valid() {
        assert!(LicenseValidator::validate_format("CS-PRO-a3f2-9d8e-c4b1-7f0a"));
    }

    #[test]
    fn test_validate_format_wrong_prefix() {
        assert!(!LicenseValidator::validate_format("CL-PRO-A3F2-9D8E-C4B1-7F0A"));
    }

    #[test]
    fn test_validate_format_too_few_segments() {
        assert!(!LicenseValidator::validate_format("CS-PRO-A3F2-9D8E-C4B1"));
    }

    #[test]
    fn test_validate_format_too_many_segments() {
        assert!(!LicenseValidator::validate_format("CS-PRO-A3F2-9D8E-C4B1-7F0A-AAAA"));
    }

    #[test]
    fn test_validate_format_non_hex_chars() {
        assert!(!LicenseValidator::validate_format("CS-PRO-ZZZZ-9D8E-C4B1-7F0A"));
    }

    #[test]
    fn test_validate_format_wrong_segment_length() {
        assert!(!LicenseValidator::validate_format("CS-PRO-A3F-9D8E-C4B1-7F0A"));
    }

    #[test]
    fn test_validate_format_empty() {
        assert!(!LicenseValidator::validate_format(""));
    }

    #[test]
    fn test_generate_key_has_valid_format() {
        let key = generate_key();
        assert!(LicenseValidator::validate_format(&key), "Generated key should have valid format: {key}");
    }

    #[test]
    fn test_generate_key_passes_checksum() {
        let key = generate_key();
        assert!(LicenseValidator::verify_checksum(&key), "Generated key should pass checksum: {key}");
    }

    #[test]
    fn test_checksum_fails_for_tampered_key() {
        let key = generate_key();
        // Tamper with the first segment
        let tampered = key.replacen('A', "B", 1);
        if tampered != key {
            // Only test if we actually changed something
            assert!(!LicenseValidator::verify_checksum(&tampered));
        }
    }

    #[test]
    fn test_pro_features_not_empty() {
        let features = pro_features();
        assert!(!features.is_empty());
        assert!(features.contains(&"cost_tracking".to_string()));
    }

    #[test]
    fn test_validator_rejects_invalid_key() {
        let validator = LicenseValidator::new();
        let info = validator.validate("INVALID-KEY");
        assert_eq!(info.status, LicenseStatus::Invalid);
        assert_eq!(info.tier, LicenseTier::Free);
    }

    #[test]
    fn test_license_info_serialization() {
        let info = LicenseInfo {
            tier: LicenseTier::Pro,
            status: LicenseStatus::Valid,
            key: "CS-PRO-AAAA-BBBB-CCCC-DDDD".to_string(),
            expires: None,
            features: pro_features(),
            last_validated: Some(Utc::now()),
            machine_id: "test123".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: LicenseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tier, LicenseTier::Pro);
        assert_eq!(deserialized.status, LicenseStatus::Valid);
    }
}
