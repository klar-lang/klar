//! Klar Package Manager
//!
//! Handles klar.toml manifest parsing, dependency resolution,
//! klar.lock generation, and package commands (add, remove).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

mod resolver;

pub use resolver::resolve;

#[derive(Debug, thiserror::Error)]
pub enum PkgError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("package not found: {0}")]
    NotFound(String),
    #[error("version conflict: {0}")]
    VersionConflict(String),
    #[error("no klar.toml found in {0}")]
    NoManifest(String),
}

pub type Result<T> = std::result::Result<T, PkgError>;

// ============================================================
// Manifest (klar.toml)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub project: ProjectSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: BTreeMap<String, DependencySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default, rename = "klar-version")]
    pub klar_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version string: `"1.0.0"` or `"^1.2"` or `">=1.0, <2.0"`
    Simple(String),
    /// Table form with more options
    Detailed(DetailedDep),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDep {
    pub version: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub path: Option<String>,
}

impl DependencySpec {
    pub fn version_str(&self) -> &str {
        match self {
            DependencySpec::Simple(v) => v,
            DependencySpec::Detailed(d) => d.version.as_deref().unwrap_or("*"),
        }
    }
}

impl Manifest {
    /// Load manifest from a klar.toml file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Find the klar.toml in the current directory or parents.
    pub fn find(start: &Path) -> Result<(PathBuf, Self)> {
        let mut dir = start.to_path_buf();
        loop {
            let manifest_path = dir.join("klar.toml");
            if manifest_path.exists() {
                let manifest = Self::load(&manifest_path)?;
                return Ok((manifest_path, manifest));
            }
            if !dir.pop() {
                break;
            }
        }
        Err(PkgError::NoManifest(start.display().to_string()))
    }

    /// Save manifest to a klar.toml file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

// ============================================================
// Lock file (klar.lock)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }

    /// Load lock file from klar.lock.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let lock: LockFile = toml::from_str(&content)?;
        Ok(lock)
    }

    /// Save lock file to klar.lock.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a deterministic checksum for the lock file content.
    pub fn checksum(&self) -> String {
        let mut hasher = Sha256::new();
        for pkg in &self.packages {
            hasher.update(pkg.name.as_bytes());
            hasher.update(b"@");
            hasher.update(pkg.version.as_bytes());
            hasher.update(b"\n");
        }
        format!("{:x}", hasher.finalize())
    }
}

// ============================================================
// Version parsing
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

impl SemVer {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches(['=', '^', '~', '>', '<', ' ']);
        let parts: Vec<&str> = s.splitn(2, '-').collect();
        let version_parts: Vec<&str> = parts[0].split('.').collect();

        let major = version_parts.first()?.parse().ok()?;
        let minor = version_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = version_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let pre = parts.get(1).map(|s| s.to_string());

        Some(Self { major, minor, patch, pre })
    }

    pub fn to_string(&self) -> String {
        if let Some(pre) = &self.pre {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    /// Check if this version satisfies a version requirement.
    pub fn satisfies(&self, req: &str) -> bool {
        let req = req.trim();
        if req == "*" {
            return true;
        }

        if req.starts_with('^') {
            // Caret: compatible with (same major, minor >= specified)
            if let Some(base) = SemVer::parse(&req[1..]) {
                return self.major == base.major
                    && (self.minor > base.minor
                        || (self.minor == base.minor && self.patch >= base.patch));
            }
        } else if req.starts_with('~') {
            // Tilde: same major.minor
            if let Some(base) = SemVer::parse(&req[1..]) {
                return self.major == base.major
                    && self.minor == base.minor
                    && self.patch >= base.patch;
            }
        } else if req.starts_with(">=") {
            if let Some(base) = SemVer::parse(&req[2..]) {
                return *self >= base;
            }
        } else if req.starts_with('>') {
            if let Some(base) = SemVer::parse(&req[1..]) {
                return *self > base;
            }
        } else if req.starts_with("<=") {
            if let Some(base) = SemVer::parse(&req[2..]) {
                return *self <= base;
            }
        } else if req.starts_with('<') {
            if let Some(base) = SemVer::parse(&req[1..]) {
                return *self < base;
            }
        } else if req.starts_with('=') {
            if let Some(base) = SemVer::parse(&req[1..]) {
                return *self == base;
            }
        } else {
            // Exact match or caret-style
            if let Some(base) = SemVer::parse(req) {
                return self.major == base.major
                    && self.minor == base.minor
                    && self.patch == base.patch;
            }
        }

        true // Unknown constraint, accept
    }
}

// ============================================================
// Commands
// ============================================================

/// Add a dependency to klar.toml.
pub fn add_dependency(manifest_path: &Path, name: &str, version: &str) -> Result<()> {
    let mut manifest = Manifest::load(manifest_path)?;
    manifest
        .dependencies
        .insert(name.to_string(), DependencySpec::Simple(version.to_string()));
    manifest.save(manifest_path)?;

    // Regenerate lock file
    let lock_path = manifest_path.with_file_name("klar.lock");
    let lock = generate_lock(&manifest)?;
    lock.save(&lock_path)?;

    Ok(())
}

/// Remove a dependency from klar.toml.
pub fn remove_dependency(manifest_path: &Path, name: &str) -> Result<()> {
    let mut manifest = Manifest::load(manifest_path)?;
    if manifest.dependencies.remove(name).is_none() {
        return Err(PkgError::NotFound(name.to_string()));
    }
    manifest.save(manifest_path)?;

    // Regenerate lock file
    let lock_path = manifest_path.with_file_name("klar.lock");
    let lock = generate_lock(&manifest)?;
    lock.save(&lock_path)?;

    Ok(())
}

/// Generate a lock file from a manifest.
pub fn generate_lock(manifest: &Manifest) -> Result<LockFile> {
    let mut lock = LockFile::new();

    for (name, spec) in &manifest.dependencies {
        let version = spec.version_str().to_string();
        let source = match spec {
            DependencySpec::Detailed(d) if d.git.is_some() => {
                format!("git+{}", d.git.as_ref().unwrap())
            }
            DependencySpec::Detailed(d) if d.path.is_some() => {
                format!("path:{}", d.path.as_ref().unwrap())
            }
            _ => format!("registry:https://pkg.klar-lang.org/{}/{}", name, version),
        };

        // Generate deterministic checksum
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(b"@");
        hasher.update(version.as_bytes());
        let checksum = format!("sha256:{:x}", hasher.finalize());

        lock.packages.push(LockedPackage {
            name: name.clone(),
            version: version.trim_start_matches('^').to_string(),
            source,
            checksum,
            dependencies: vec![],
        });
    }

    // Sort for determinism
    lock.packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(lock)
}

/// Initialize a new klar.toml in the given directory.
pub fn init(dir: &Path, name: &str) -> Result<PathBuf> {
    let manifest = Manifest {
        project: ProjectSection {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: None,
            authors: vec![],
            license: Some("Apache-2.0".to_string()),
            repository: None,
            homepage: None,
            klar_version: Some("0.1.0".to_string()),
        },
        dependencies: BTreeMap::new(),
        dev_dependencies: BTreeMap::new(),
    };

    let manifest_path = dir.join("klar.toml");
    manifest.save(&manifest_path)?;

    Ok(manifest_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_manifest() {
        let toml_str = r#"
[project]
name = "my-app"
version = "1.0.0"

[dependencies]
http-server = "^1.0"
json-utils = { version = "2.1.0", git = "https://github.com/example/json-utils" }
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.project.name, "my-app");
        assert_eq!(manifest.dependencies.len(), 2);
        assert!(manifest.dependencies.contains_key("http-server"));
    }

    #[test]
    fn semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn semver_satisfies_caret() {
        let v = SemVer::parse("1.3.0").unwrap();
        assert!(v.satisfies("^1.2"));
        assert!(!v.satisfies("^2.0"));
    }

    #[test]
    fn semver_satisfies_tilde() {
        let v = SemVer::parse("1.2.5").unwrap();
        assert!(v.satisfies("~1.2.0"));
        assert!(!v.satisfies("~1.3.0"));
    }

    #[test]
    fn lock_deterministic() {
        let manifest: Manifest = toml::from_str(r#"
[project]
name = "test"
version = "0.1.0"

[dependencies]
alpha = "1.0.0"
beta = "2.0.0"
"#).unwrap();

        let lock1 = generate_lock(&manifest).unwrap();
        let lock2 = generate_lock(&manifest).unwrap();
        assert_eq!(lock1.checksum(), lock2.checksum());
        assert_eq!(lock1.packages[0].name, "alpha"); // sorted
    }

    #[test]
    fn add_and_remove() {
        let dir = std::env::temp_dir().join("klar_pkg_test");
        let _ = fs::create_dir_all(&dir);

        let manifest_path = init(&dir, "test-project").unwrap();
        add_dependency(&manifest_path, "http-server", "^1.0").unwrap();

        let manifest = Manifest::load(&manifest_path).unwrap();
        assert!(manifest.dependencies.contains_key("http-server"));

        remove_dependency(&manifest_path, "http-server").unwrap();
        let manifest = Manifest::load(&manifest_path).unwrap();
        assert!(!manifest.dependencies.contains_key("http-server"));

        let _ = fs::remove_dir_all(&dir);
    }
}
