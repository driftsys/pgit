use serde::{Deserialize, Serialize};

use crate::error::PgitError;

// ── Registry manifest (registry.toml) ────────────────────────────────────────

/// Top-level registry.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryManifest {
    pub registry: RegistryMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMeta {
    pub name:        String,
    /// Spec version, e.g. "pgit/1"
    pub format:      String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mode:        RegistryMode,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryMode {
    /// Versioned distribution — yanking allowed.
    #[default]
    Distribution,
    /// Append-only regulatory archive — no overwrites, ever.
    Archive,
}

impl RegistryManifest {
    pub fn parse(toml_str: &str) -> Result<Self, PgitError> {
        toml::from_str(toml_str).map_err(|e| PgitError::InvalidManifest(e.to_string()))
    }
}

// ── Package manifest (package.toml) ──────────────────────────────────────────

/// Top-level package.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageMeta,
    #[serde(default)]
    pub files:   FilesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name:        String,
    pub version:     String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors:     Vec<String>,
    #[serde(default)]
    pub license:     Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_include() -> Vec<String> {
    vec!["**/*".to_string()]
}

impl PackageManifest {
    pub fn parse(toml_str: &str) -> Result<Self, PgitError> {
        toml::from_str(toml_str).map_err(|e| PgitError::InvalidManifest(e.to_string()))
    }
}

// ── Provenance record (provenance.json) ──────────────────────────────────────

/// Written alongside every published package version; immutable after creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub purl:         String,
    pub resolved_sha: String,
    pub content_hash: String,
    pub published_at: String,  // RFC 3339
    pub publisher:    Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_registry_manifest() {
        let toml = r#"
[registry]
name = "my-registry"
format = "pgit/1"
description = "test"
mode = "distribution"
"#;
        let m = RegistryManifest::parse(toml).unwrap();
        assert_eq!(m.registry.name, "my-registry");
        assert_eq!(m.registry.mode, RegistryMode::Distribution);
    }

    #[test]
    fn parse_package_manifest() {
        let toml = r#"
[package]
name = "my-tool"
version = "1.0.0"
description = "A useful tool"
authors = ["Alice"]
"#;
        let m = PackageManifest::parse(toml).unwrap();
        assert_eq!(m.package.name, "my-tool");
        assert_eq!(m.package.version, "1.0.0");
        assert_eq!(m.files.include, vec!["**/*"]);
    }

    #[test]
    fn registry_mode_defaults_to_distribution() {
        let toml = "[registry]\nname = \"r\"\nformat = \"pgit/1\"\n";
        let m = RegistryManifest::parse(toml).unwrap();
        assert_eq!(m.registry.mode, RegistryMode::Distribution);
    }
}
