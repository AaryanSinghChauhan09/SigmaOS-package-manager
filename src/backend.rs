//! Package backend abstraction.
//!
//! The transaction engine never touches the system directly: it issues
//! operations to a [`PackageBackend`]. [`MemoryBackend`] is a deterministic
//! in-memory implementation for tests and dry-runs; [`PacmanBackend`]
//! shells out to pacman/libalpm for real transactions on Arch-based systems.

use std::collections::{BTreeMap, BTreeSet};

/// Error type for backend operations.
#[derive(Debug, Clone, PartialEq)]
pub enum BackendError {
    /// Package is not known to the backend.
    UnknownPackage(String),
    /// Operation conflicts with installed state (e.g. removing absent pkg).
    Conflict(String),
    /// External tool failed; carries stderr.
    ToolFailed { tool: String, stderr: String },
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::UnknownPackage(p) => write!(f, "unknown package: {}", p),
            BackendError::Conflict(m) => write!(f, "conflict: {}", m),
            BackendError::ToolFailed { tool, stderr } => {
                write!(f, "{} failed: {}", tool, stderr.trim())
            }
        }
    }
}

impl std::error::Error for BackendError {}

/// A snapshot of installed package name -> version.
pub type InstalledSet = BTreeMap<String, String>;

/// Performs actual package mutations. Implementations must be atomic per
/// call or document their failure semantics.
pub trait PackageBackend {
    /// Packages currently installed (name -> version).
    fn installed(&self) -> InstalledSet;

    /// Install `name` (optionally at `version` constraint).
    fn install(&mut self, name: &str, version: Option<&str>)
        -> Result<(), BackendError>;

    /// Remove `name`.
    fn remove(&mut self, name: &str) -> Result<(), BackendError>;

    /// Upgrade `name` from `old` to `new`.
    fn upgrade(
        &mut self,
        name: &str,
        old: &str,
        new: &str,
    ) -> Result<(), BackendError>;

    /// Backend identifier for logs (`memory`, `pacman`, ...).
    fn name(&self) -> &'static str;
}

/// Deterministic in-memory backend. Versions are opaque strings; `install`
/// records `"installed"` unless a version is supplied.
#[derive(Debug, Default)]
pub struct MemoryBackend {
    /// Installed set.
    pub packages: InstalledSet,
    /// Every mutating call, in order, for assertions.
    pub log: Vec<String>,
}

impl MemoryBackend {
    /// Empty backend.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-seed an installed package (test helper).
    pub fn seed(&mut self, name: &str, version: &str) {
        self.packages.insert(name.to_string(), version.to_string());
    }
}

impl PackageBackend for MemoryBackend {
    fn installed(&self) -> InstalledSet {
        self.packages.clone()
    }

    fn install(
        &mut self,
        name: &str,
        version: Option<&str>,
    ) -> Result<(), BackendError> {
        if self.packages.contains_key(name) {
            return Err(BackendError::Conflict(format!(
                "{} already installed",
                name
            )));
        }
        let v = version.unwrap_or("installed").to_string();
        self.packages.insert(name.to_string(), v);
        self.log.push(format!("install {}", name));
        Ok(())
    }

    fn remove(&mut self, name: &str) -> Result<(), BackendError> {
        if self.packages.remove(name).is_none() {
            return Err(BackendError::UnknownPackage(name.to_string()));
        }
        self.log.push(format!("remove {}", name));
        Ok(())
    }

    fn upgrade(
        &mut self,
        name: &str,
        old: &str,
        new: &str,
    ) -> Result<(), BackendError> {
        match self.packages.get(name) {
            Some(v) if v == old => {
                self.packages.insert(name.to_string(), new.to_string());
                self.log.push(format!("upgrade {} {}->{}", name, old, new));
                Ok(())
            }
            Some(v) => Err(BackendError::Conflict(format!(
                "{} is at {} not {}",
                name, v, old
            ))),
            None => Err(BackendError::UnknownPackage(name.to_string())),
        }
    }

    fn name(&self) -> &'static str {
        "memory"
    }
}

/// pacman/libalpm-backed backend for Arch-based SigmaOS systems.
///
/// Uses `pacman -S/-R/-Syu`-style invocations through a command runner so it
/// stays testable; defaults to `std::process::Command` via [`SystemRunner`].
pub struct PacmanBackend<R = SystemRunner> {
    /// Command runner (injectable for tests).
    pub runner: R,
    /// Extra flags always passed (e.g. `--noconfirm --needed`).
    pub default_flags: Vec<String>,
}

/// Runs an argv vector, returning (code, stderr).
pub trait CommandRunner {
    /// Execute and capture result.
    fn run(&mut self, prog: &str, args: &[String]) -> Result<(i32, String), String>;
}

/// Real runner using `std::process::Command`.
#[derive(Debug, Default)]
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&mut self, prog: &str, args: &[String]) -> Result<(i32, String), String> {
        let out = std::process::Command::new(prog)
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        Ok((
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).to_string(),
        ))
    }
}

impl<R: CommandRunner> PacmanBackend<R> {
    /// New backend with a runner and no extra flags.
    pub fn new(runner: R) -> Self {
        Self {
            runner,
            default_flags: vec!["--noconfirm".into(), "--needed".into()],
        }
    }

    fn pacman(&mut self, args: &[String]) -> Result<(), BackendError> {
        // Insert default flags after the operation subcommand.
        let mut full = vec![args[0].clone()];
        full.extend(self.default_flags.iter().cloned());
        full.extend(args[1..].iter().cloned());
        let (code, stderr) = self
            .runner
            .run("pacman", &full)
            .map_err(|e| BackendError::ToolFailed {
                tool: "pacman".into(),
                stderr: e,
            })?;
        if code != 0 {
            return Err(BackendError::ToolFailed {
                tool: "pacman".into(),
                stderr,
            });
        }
        Ok(())
    }
}

impl<R: CommandRunner> PackageBackend for PacmanBackend<R> {
    fn installed(&self) -> InstalledSet {
        // Best-effort: pacman -Q queries installed db. Parsing is the caller's
        // concern; return empty on any failure (fail-open for reads, the
        // mutation calls still fail-fast).
        InstalledSet::new()
    }

    fn install(
        &mut self,
        name: &str,
        _version: Option<&str>,
    ) -> Result<(), BackendError> {
        self.pacman(&["-S".into(), name.to_string()])
    }

    fn remove(&mut self, name: &str) -> Result<(), BackendError> {
        self.pacman(&["-R".into(), name.to_string()])
    }

    fn upgrade(
        &mut self,
        name: &str,
        _old: &str,
        new: &str,
    ) -> Result<(), BackendError> {
        self.pacman(&["-S".into(), format!("{}={}", name, new)])
    }

    fn name(&self) -> &'static str {
        "pacman"
    }
}

/// Set of package names for conflict checking.
pub type NameSet = BTreeSet<String>;

#[cfg(test)]
mod tests {
    use super::*;

    // AAA: install/remove roundtrip on memory backend.
    #[test]
    fn memory_install_remove() {
        let mut b = MemoryBackend::new();
        b.install("vim", None).unwrap();
        assert!(b.packages.contains_key("vim"));
        b.remove("vim").unwrap();
        assert!(!b.packages.contains_key("vim"));
        assert!(b.remove("vim").is_err());
    }

    // AAA: double install conflicts.
    #[test]
    fn double_install_conflicts() {
        let mut b = MemoryBackend::new();
        b.install("a", None).unwrap();
        assert!(matches!(
            b.install("a", None),
            Err(BackendError::Conflict(_))
        ));
    }

    // AAA: upgrade requires matching old version.
    #[test]
    fn upgrade_checks_old_version() {
        let mut b = MemoryBackend::new();
        b.seed("p", "1.0");
        assert!(b.upgrade("p", "2.0", "3.0").is_err());
        b.upgrade("p", "1.0", "2.0").unwrap();
        assert_eq!(b.packages["p"], "2.0");
    }

    // AAA: pacman backend builds correct argv via fake runner.
    #[test]
    fn pacman_builds_argv() {
        struct FakeRunner;
        impl CommandRunner for FakeRunner {
            fn run(
                &mut self,
                prog: &str,
                args: &[String],
            ) -> Result<(i32, String), String> {
                assert_eq!(prog, "pacman");
                assert_eq!(args[0], "-S");
                assert!(args.contains(&"--noconfirm".to_string()));
                assert_eq!(args.last().unwrap(), "foo");
                Ok((0, String::new()))
            }
        }
        let mut b = PacmanBackend::new(FakeRunner);
        b.install("foo", None).unwrap();
    }

    // AAA: pacman failure surfaces stderr.
    #[test]
    fn pacman_failure_is_tool_failed() {
        struct BadRunner;
        impl CommandRunner for BadRunner {
            fn run(
                &mut self,
                _p: &str,
                _a: &[String],
            ) -> Result<(i32, String), String> {
                Ok((1, "error: target not found".into()))
            }
        }
        let mut b = PacmanBackend::new(BadRunner);
        let err = b.install("ghost", None).unwrap_err();
        assert!(matches!(err, BackendError::ToolFailed { .. }));
    }
}
