mod storage;
mod verify;

pub use storage::LicenseStorage;
pub use verify::{LicenseInfo, LicenseStatus, LicenseTier, LicenseValidator};

/// Check whether Pro features are currently available.
/// Returns the license info if valid, None otherwise.
pub fn check_pro() -> Option<LicenseInfo> {
    let storage = LicenseStorage::new();
    let key = storage.load_key()?;
    let validator = LicenseValidator::new();
    let info = validator.validate(&key);
    if info.status == LicenseStatus::Valid {
        Some(info)
    } else {
        None
    }
}

/// Returns true if Pro features should be enabled.
pub fn is_pro() -> bool {
    check_pro().is_some()
}
