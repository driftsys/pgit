use std::fmt;

use crate::error::PgitError;

/// A Package URL (PURL) — https://github.com/package-url/purl-spec
///
/// pgit canonical form:
///   `pkg:pgit/github.com/owner/repo/package-name@version`
///
/// Components:
///   type      = "pgit"
///   namespace = "github.com/owner/repo"   (the registry)
///   name      = "package-name"            (package within the registry)
///   version   = semver or git ref
///   subpath   = optional path within the package
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purl {
    pub pkg_type:  String,
    pub namespace: Option<String>,
    pub name:      String,
    pub version:   Option<String>,
    pub subpath:   Option<String>,
}

impl Purl {
    /// Parse a PURL string.
    pub fn parse(s: &str) -> Result<Self, PgitError> {
        let err = || PgitError::InvalidPurl(s.to_string());

        let rest = s.strip_prefix("pkg:").ok_or_else(err)?;

        // split off #subpath
        let (rest, subpath) = match rest.split_once('#') {
            Some((before, sub)) if !sub.is_empty() => (before, Some(sub.to_string())),
            Some(_) => return Err(err()),
            None => (rest, None),
        };

        // split off @version
        let (rest, version) = match rest.split_once('@') {
            Some((before, ver)) if !ver.is_empty() => (before, Some(ver.to_string())),
            Some(_) => return Err(err()),
            None => (rest, None),
        };

        // split type from the rest
        let (pkg_type, path) = rest.split_once('/').ok_or_else(err)?;
        if pkg_type.is_empty() || path.is_empty() {
            return Err(err());
        }

        // last path segment = name; everything before = namespace
        let (namespace, name) = match path.rsplit_once('/') {
            Some((ns, n)) if !ns.is_empty() && !n.is_empty() => {
                (Some(ns.to_string()), n.to_string())
            }
            Some(_) => return Err(err()),
            None => (None, path.to_string()),
        };

        Ok(Purl { pkg_type: pkg_type.to_string(), namespace, name, version, subpath })
    }

    /// Return the registry namespace (host + owner + repo), if present.
    pub fn registry(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

impl fmt::Display for Purl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pkg:{}/", self.pkg_type)?;
        if let Some(ns) = &self.namespace {
            write!(f, "{}/", ns)?;
        }
        write!(f, "{}", self.name)?;
        if let Some(ver) = &self.version {
            write!(f, "@{}", ver)?;
        }
        if let Some(sub) = &self.subpath {
            write!(f, "#{}", sub)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_pgit_purl() {
        let p = Purl::parse("pkg:pgit/github.com/owner/repo/my-tool@1.0.0").unwrap();
        assert_eq!(p.pkg_type, "pgit");
        assert_eq!(p.namespace.as_deref(), Some("github.com/owner/repo"));
        assert_eq!(p.name, "my-tool");
        assert_eq!(p.version.as_deref(), Some("1.0.0"));
        assert_eq!(p.subpath, None);
    }

    #[test]
    fn parse_purl_with_subpath() {
        let p = Purl::parse("pkg:pgit/github.com/owner/repo/tool@2.0.0#scripts").unwrap();
        assert_eq!(p.subpath.as_deref(), Some("scripts"));
    }

    #[test]
    fn parse_no_namespace() {
        let p = Purl::parse("pkg:pgit/standalone@0.1.0").unwrap();
        assert_eq!(p.namespace, None);
        assert_eq!(p.name, "standalone");
    }

    #[test]
    fn roundtrip() {
        let raw = "pkg:pgit/github.com/owner/repo/my-tool@1.0.0";
        assert_eq!(Purl::parse(raw).unwrap().to_string(), raw);
    }

    #[test]
    fn reject_missing_pkg_prefix() {
        assert!(Purl::parse("pgit/owner/repo/tool@1.0").is_err());
    }

    #[test]
    fn reject_empty_version() {
        assert!(Purl::parse("pkg:pgit/owner/repo/tool@").is_err());
    }
}
