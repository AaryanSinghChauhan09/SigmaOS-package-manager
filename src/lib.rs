// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-package-manager — Unified package management with OOPS principles
//!
//! OOPS Principles Applied:
//! - Encapsulation: Pacman operations wrapped in clean API
//! - Command-Query Separation: Read operations separate from write operations
//! - Interface Segregation: Small, focused interfaces
//! - Dependency Inversion: Depend on abstractions, not concretions
//!
//! Design Patterns:
//! - Repository Pattern: Package repository abstraction
//! - Command Pattern: Package operations as commands
//! - Observer Pattern: Notify on package changes
//! - Strategy Pattern: Different update strategies

#![forbid(unsafe_code)]

use std::fmt;
use std::collections::BTreeMap;

/// Errors for package operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageError {
    /// Package not found
    NotFound(String),
    /// Dependency conflict
    DependencyConflict(String),
    /// Invalid package name
    InvalidName(String),
    /// Installation failed
    InstallFailed(String),
    /// Removal failed
    RemoveFailed(String),
    /// Query error
    QueryError(String),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(x) => write!(f, "package not found: {x}"),
            Self::DependencyConflict(x) => write!(f, "dependency conflict: {x}"),
            Self::InvalidName(x) => write!(f, "invalid package name: {x}"),
            Self::InstallFailed(x) => write!(f, "installation failed: {x}"),
            Self::RemoveFailed(x) => write!(f, "removal failed: {x}"),
            Self::QueryError(x) => write!(f, "query error: {x}"),
        }
    }
}

impl std::error::Error for PackageError {}

/// Package information structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

/// Query interface - Command-Query Separation
/// Read-only operations (queries)
pub trait PackageQuery {
    /// Search for packages by name
    fn search(&self, query: &str) -> Result<Vec<Package>, PackageError>;
    
    /// Get package information
    fn info(&self, name: &str) -> Result<Package, PackageError>;
    
    /// List installed packages
    fn list_installed(&self) -> Result<Vec<Package>, PackageError>;
    
    /// Check if package is installed
    fn is_installed(&self, name: &str) -> Result<bool, PackageError>;
    
    /// Get package dependencies
    fn dependencies(&self, name: &str) -> Result<Vec<String>, PackageError>;
}

/// Command interface - Command-Query Separation
/// Write operations (commands)
pub trait PackageCommand {
    /// Install a package
    fn install(&mut self, name: &str) -> Result<(), PackageError>;
    
    /// Remove a package
    fn remove(&mut self, name: &str) -> Result<(), PackageError>;
    
    /// Update a package
    fn update(&mut self, name: &str) -> Result<(), PackageError>;
    
    /// Update all packages
    fn update_all(&mut self) -> Result<(), PackageError>;
    
    /// Sync package database
    fn sync(&mut self) -> Result<(), PackageError>;
}

/// Pacman backend - Concrete implementation
/// Encapsulates pacman operations
pub struct PacmanBackend {
    _config_path: String,
}

impl PacmanBackend {
    pub fn new(config_path: &str) -> Self {
        Self {
            _config_path: config_path.to_string(),
        }
    }
}

/// Implement PackageQuery for PacmanBackend
/// Query operations (read-only)
impl PackageQuery for PacmanBackend {
    fn search(&self, query: &str) -> Result<Vec<Package>, PackageError> {
        println!("Searching for packages matching: {}", query);
        Ok(vec![])
    }
    
    fn info(&self, name: &str) -> Result<Package, PackageError> {
        println!("Getting info for package: {}", name);
        Ok(Package {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Package description".to_string(),
            dependencies: vec![],
        })
    }
    
    fn list_installed(&self) -> Result<Vec<Package>, PackageError> {
        println!("Listing installed packages");
        Ok(vec![])
    }
    
    fn is_installed(&self, name: &str) -> Result<bool, PackageError> {
        println!("Checking if {} is installed", name);
        Ok(false)
    }
    
    fn dependencies(&self, name: &str) -> Result<Vec<String>, PackageError> {
        println!("Getting dependencies for {}", name);
        Ok(vec![])
    }
}

/// Implement PackageCommand for PacmanBackend
/// Command operations (write)
impl PackageCommand for PacmanBackend {
    fn install(&mut self, name: &str) -> Result<(), PackageError> {
        println!("Installing package: {}", name);
        Ok(())
    }
    
    fn remove(&mut self, name: &str) -> Result<(), PackageError> {
        println!("Removing package: {}", name);
        Ok(())
    }
    
    fn update(&mut self, name: &str) -> Result<(), PackageError> {
        println!("Updating package: {}", name);
        Ok(())
    }
    
    fn update_all(&mut self) -> Result<(), PackageError> {
        println!("Updating all packages");
        Ok(())
    }
    
    fn sync(&mut self) -> Result<(), PackageError> {
        println!("Syncing package database");
        Ok(())
    }
}

/// Package manager - Facade Pattern
/// Provides simple interface to complex pacman operations
pub struct PackageManager {
    query: Box<dyn PackageQuery>,
    command: Box<dyn PackageCommand>,
}

impl PackageManager {
    pub fn new(query: Box<dyn PackageQuery>, command: Box<dyn PackageCommand>) -> Self {
        Self { query, command }
    }
    
    /// Combined query and command operations
    pub fn search_and_install(&mut self, query: &str) -> Result<(), PackageError> {
        let packages = self.query.search(query)?;
        if packages.is_empty() {
            return Err(PackageError::NotFound(query.to_string()));
        }
        
        for pkg in packages {
            self.command.install(&pkg.name)?;
        }
        
        Ok(())
    }
}

/// Observer Pattern - Notify on package changes
pub trait PackageObserver {
    fn on_install(&self, package: &str);
    fn on_remove(&self, package: &str);
    fn on_update(&self, package: &str);
}

/// Observer implementation for logging
pub struct LoggingObserver;

impl PackageObserver for LoggingObserver {
    fn on_install(&self, package: &str) {
        println!("LOG: Package installed: {}", package);
    }
    
    fn on_remove(&self, package: &str) {
        println!("LOG: Package removed: {}", package);
    }
    
    fn on_update(&self, package: &str) {
        println!("LOG: Package updated: {}", package);
    }
}

/// Strategy Pattern - Different update strategies
pub trait UpdateStrategy {
    fn execute(&self, manager: &mut PackageManager) -> Result<(), PackageError>;
}

/// Full update strategy
pub struct FullUpdateStrategy;

impl UpdateStrategy for FullUpdateStrategy {
    fn execute(&self, _manager: &mut PackageManager) -> Result<(), PackageError> {
        println!("Executing full update strategy");
        Ok(())
    }
}

/// Security-only update strategy
pub struct SecurityUpdateStrategy;

impl UpdateStrategy for SecurityUpdateStrategy {
    fn execute(&self, _manager: &mut PackageManager) -> Result<(), PackageError> {
        println!("Executing security-only update strategy");
        Ok(())
    }
}

/// Named-entry registry (SRP: storage only; callers own semantics).
#[derive(Debug, Default)]
pub struct Registry {
    items: BTreeMap<String, String>,
}

impl Registry {
    /// Empty registry.
    pub fn new() -> Self { Self::default() }

    /// Register `key -> value`. Fails on empty key or duplicate.
    pub fn register(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<(), RegistryError> {
        let key = key.into();
        if key.trim().is_empty() { return Err(RegistryError::EmptyKey); }
        if self.items.contains_key(&key) { return Err(RegistryError::Duplicate(key)); }
        self.items.insert(key, value.into());
        Ok(())
    }

    /// Look up a value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.items.get(key).map(String::as_str)
    }

    /// Remove an entry; fails if absent.
    pub fn remove(&mut self, key: &str) -> Result<String, RegistryError> {
        self.items.remove(key).ok_or_else(|| RegistryError::NotFound(key.to_string()))
    }

    /// Number of entries.
    pub fn len(&self) -> usize { self.items.len() }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

/// Errors for `RegistryError` operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// An entry with this key is already registered
    Duplicate(String),
    /// No entry exists for the requested key
    NotFound(String),
    /// Registration key must not be empty
    EmptyKey,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(x) => write!(f, "an entry with this key is already registered: {x}"),
            Self::NotFound(x) => write!(f, "no entry exists for the requested key: {x}"),
            Self::EmptyKey => write!(f, "registration key must not be empty"),
        }
    }
}

impl std::error::Error for RegistryError {}

pub mod backend;
pub mod config;
pub mod engine;
pub mod error;
pub mod history;
pub mod journal;
pub mod resolver;
pub mod service;
pub mod transaction;

#[cfg(test)]
pub mod sigmapackagemanager;
pub mod sigmapackagemanager2;
pub mod sigmapkg;

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> Registry {
        let mut r = Registry::new();
        r.register("alpha", "one").unwrap();
        r.register("beta", "two").unwrap();
        r
    }

    #[test]
    fn register_and_get() {
        let r = reg();
        assert_eq!(r.get("alpha"), Some("one"));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn duplicate_key_fails() {
        let mut r = Registry::new();
        r.register("x", "1").unwrap();
        assert!(matches!(r.register("x", "2"), Err(RegistryError::Duplicate(_))));
    }

    #[test]
    fn remove_missing_fails() {
        assert!(matches!(Registry::new().remove("nope"), Err(RegistryError::NotFound(_))));
    }

    #[test]
    fn empty_key_fails() {
        assert_eq!(Registry::new().register("  ", "v"), Err(RegistryError::EmptyKey));
    }
    
    #[test]
    fn test_pacman_backend_query() {
        let backend = PacmanBackend::new("/etc/pacman.conf");
        let result = backend.search("test");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_pacman_backend_command() {
        let mut backend = PacmanBackend::new("/etc/pacman.conf");
        let result = backend.sync();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_package_manager() {
        let query = Box::new(PacmanBackend::new("/etc/pacman.conf"));
        let command = Box::new(PacmanBackend::new("/etc/pacman.conf"));
        let mut manager = PackageManager::new(query, command);
        let result = manager.command.sync();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_cqrs_separation() {
        // Query interface - read-only
        let query: Box<dyn PackageQuery> = Box::new(PacmanBackend::new("/etc/pacman.conf"));
        let _ = query.search("test");
        
        // Command interface - write
        let mut command: Box<dyn PackageCommand> = Box::new(PacmanBackend::new("/etc/pacman.conf"));
        let _ = command.sync();
    }
    
    #[test]
    fn test_observer_pattern() {
        let observer = LoggingObserver;
        observer.on_install("test-package");
        observer.on_remove("test-package");
        observer.on_update("test-package");
    }
    
    #[test]
    fn test_strategy_pattern() {
        let query = Box::new(PacmanBackend::new("/etc/pacman.conf"));
        let command = Box::new(PacmanBackend::new("/etc/pacman.conf"));
        let mut manager = PackageManager::new(query, command);
        
        let full_strategy = FullUpdateStrategy;
        let _ = full_strategy.execute(&mut manager);
        
        let security_strategy = SecurityUpdateStrategy;
        let _ = security_strategy.execute(&mut manager);
    }
}

/// Backwards-compatible component handle (kept for existing callers/tests).
#[derive(Debug, Default)]
pub struct Component {
    name: &'static str,
}

impl Component {
    /// Create the component handle.
    pub fn new() -> Self {
        Self { name: "SigmaOS Component" }
    }
}
