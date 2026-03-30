//! Simplified dependency resolver using a greedy algorithm.
//! Will be upgraded to PubGrub in a future sprint.

use crate::{DependencySpec, LockFile, LockedPackage, Manifest, PkgError, Result, SemVer};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

/// Resolve all dependencies for a manifest, producing a lock file.
pub fn resolve(manifest: &Manifest) -> Result<LockFile> {
    let mut resolved: BTreeMap<String, LockedPackage> = BTreeMap::new();
    let mut visited: HashSet<String> = HashSet::new();

    // Resolve direct dependencies
    for (name, spec) in &manifest.dependencies {
        resolve_package(name, spec, &mut resolved, &mut visited)?;
    }

    let mut lock = LockFile::new();
    lock.packages = resolved.into_values().collect();
    lock.packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(lock)
}

fn resolve_package(
    name: &str,
    spec: &DependencySpec,
    resolved: &mut BTreeMap<String, LockedPackage>,
    visited: &mut HashSet<String>,
) -> Result<()> {
    if visited.contains(name) {
        return Ok(()); // Already resolved
    }
    visited.insert(name.to_string());

    let version = spec.version_str().trim_start_matches('^').to_string();

    let source = match spec {
        DependencySpec::Detailed(d) if d.git.is_some() => {
            format!("git+{}", d.git.as_ref().unwrap())
        }
        DependencySpec::Detailed(d) if d.path.is_some() => {
            format!("path:{}", d.path.as_ref().unwrap())
        }
        _ => format!("registry:https://pkg.klar-lang.org/{}/{}", name, version),
    };

    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b"@");
    hasher.update(version.as_bytes());
    let checksum = format!("sha256:{:x}", hasher.finalize());

    resolved.insert(
        name.to_string(),
        LockedPackage {
            name: name.to_string(),
            version,
            source,
            checksum,
            dependencies: vec![],
        },
    );

    Ok(())
}
