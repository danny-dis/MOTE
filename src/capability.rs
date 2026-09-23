//! Capability construction and validation.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCapability(pub String);

impl fmt::Display for UnknownCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown capability: {}", self.0)
    }
}

impl Error for UnknownCapability {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityRegistry {
    names: BTreeSet<String>,
}

impl CapabilityRegistry {
    pub fn from_names(names: &[impl AsRef<str>]) -> Result<Self, UnknownCapability> {
        let mut validated = BTreeSet::new();
        for name in names.iter().map(AsRef::as_ref) {
            if !matches!(
                name,
                "shell" | "read_file" | "write_file" | "list_dir" | "git" | "decision"
            ) {
                return Err(UnknownCapability(name.to_owned()));
            }
            validated.insert(name.to_owned());
        }
        Ok(Self { names: validated })
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn allows(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceFs {
    root: PathBuf,
}

impl WorkspaceFs {
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = fs::canonicalize(root)?;
        if !root.is_dir() {
            return Err(io::Error::other("workspace root is not a directory"));
        }
        Ok(Self { root })
    }

    fn resolve(&self, input: impl AsRef<Path>, for_write: bool) -> io::Result<PathBuf> {
        let path = input.as_ref();
        if path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path escapes workspace",
            ));
        }
        let candidate = self.root.join(path);
        let resolved = if fs::symlink_metadata(&candidate).is_ok() {
            fs::canonicalize(&candidate)?
        } else if for_write {
            let parent = candidate
                .parent()
                .ok_or_else(|| io::Error::other("invalid path"))?;
            let parent = fs::canonicalize(parent)?;
            parent.join(
                candidate
                    .file_name()
                    .ok_or_else(|| io::Error::other("invalid path"))?,
            )
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "path does not exist",
            ));
        };
        if !resolved.starts_with(&self.root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path escapes workspace",
            ));
        }
        Ok(resolved)
    }

    pub(crate) fn same_file(
        &self,
        first: impl AsRef<Path>,
        second: impl AsRef<Path>,
    ) -> io::Result<bool> {
        Ok(self.resolve(first, false)? == self.resolve(second, false)?)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn read(&self, path: impl AsRef<Path>) -> io::Result<String> {
        fs::read_to_string(self.resolve(path, false)?)
    }

    pub fn write(&self, path: impl AsRef<Path>, content: &str) -> io::Result<()> {
        fs::write(self.resolve(path, true)?, content)
    }

    pub fn write_if_absent(&self, path: impl AsRef<Path>, content: &str) -> io::Result<()> {
        use std::io::Write;
        let resolved = self.resolve(path, true)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(resolved)?;
        file.write_all(content.as_bytes())
    }

    pub fn list(&self, path: impl AsRef<Path>) -> io::Result<Vec<String>> {
        fs::read_dir(self.resolve(path, false)?)?
            .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect()
    }
}
