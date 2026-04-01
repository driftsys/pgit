use semver::{Version, VersionReq};

use crate::error::PgitError;

/// Resolve the best matching version from a list of available version strings
/// against a semver requirement.
///
/// Returns the highest matching version, or an error if none match.
pub fn resolve(req_str: &str, available: &[&str]) -> Result<String, PgitError> {
    let req = VersionReq::parse(req_str)
        .map_err(|e| PgitError::VersionNotFound(format!("invalid requirement '{}': {}", req_str, e)))?;

    let mut candidates: Vec<Version> = available
        .iter()
        .filter_map(|v| Version::parse(v).ok())
        .filter(|v| req.matches(v))
        .collect();

    candidates.sort();
    candidates
        .pop()
        .map(|v| v.to_string())
        .ok_or_else(|| PgitError::VersionNotFound(format!("no version matching '{}'", req_str)))
}

/// Return true if `version` is a plain git ref (branch/tag/SHA) rather than semver.
pub fn is_git_ref(version: &str) -> bool {
    Version::parse(version).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_highest_matching() {
        let v = resolve("^1.0", &["1.0.0", "1.2.0", "2.0.0"]).unwrap();
        assert_eq!(v, "1.2.0");
    }

    #[test]
    fn resolves_exact() {
        let v = resolve("=1.0.0", &["1.0.0", "1.1.0"]).unwrap();
        assert_eq!(v, "1.0.0");
    }

    #[test]
    fn error_on_no_match() {
        let err = resolve("^3.0", &["1.0.0", "2.0.0"]).unwrap_err();
        assert!(err.to_string().contains("no version matching"));
    }

    #[test]
    fn git_ref_detection() {
        assert!(is_git_ref("main"));
        assert!(is_git_ref("abc1234"));
        assert!(!is_git_ref("1.0.0"));
    }
}
