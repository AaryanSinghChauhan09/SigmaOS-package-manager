use crate::error::PackageManagerError;
use std::collections::{HashMap, HashSet, VecDeque};

/// A simple in-memory package repository entry.
#[derive(Debug, Clone)]
pub struct RepoPkg {
    /// Package name.
    pub name: String,
    /// Direct dependencies (package names).
    pub deps: Vec<String>,
}

impl RepoPkg {
    /// Creates a new [`RepoPkg`].
    pub fn new(name: impl Into<String>, deps: Vec<&str>) -> Self {
        Self {
            name: name.into(),
            deps: deps.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Resolves package dependencies using a simple BFS-based topological sort.
#[derive(Debug, Default)]
pub struct DependencyResolver {
    repo: HashMap<String, RepoPkg>,
}

impl DependencyResolver {
    /// Creates a new empty [`DependencyResolver`].
    pub fn new() -> Self {
        Self { repo: HashMap::new() }
    }

    /// Registers a package in the resolver's repository.
    pub fn register(&mut self, pkg: RepoPkg) {
        self.repo.insert(pkg.name.clone(), pkg);
    }

    /// Returns an ordered install list for `pkg_name` (dependencies first).
    ///
    /// Returns [`PackageManagerError::CycleDetected`] if a dependency cycle exists,
    /// or [`PackageManagerError::PackageNotFound`] if a required package is absent.
    pub fn resolve(&self, pkg_name: &str) -> Result<Vec<String>, PackageManagerError> {
        let mut order: Vec<String> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut in_stack: HashSet<String> = HashSet::new();
        self.dfs(pkg_name, &mut order, &mut visited, &mut in_stack)?;
        Ok(order)
    }

    fn dfs(
        &self,
        name: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
    ) -> Result<(), PackageManagerError> {
        if in_stack.contains(name) {
            return Err(PackageManagerError::CycleDetected(name.to_string()));
        }
        if visited.contains(name) {
            return Ok(());
        }
        let pkg = self
            .repo
            .get(name)
            .ok_or_else(|| PackageManagerError::PackageNotFound(name.to_string()))?;
        in_stack.insert(name.to_string());
        for dep in &pkg.deps {
            self.dfs(dep, order, visited, in_stack)?;
        }
        in_stack.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());
        Ok(())
    }
}

/// Checks whether `name` appears before `before` in `order`.
fn comes_before(order: &[String], name: &str, before: &str) -> bool {
    let pos_name = order.iter().position(|x| x == name);
    let pos_before = order.iter().position(|x| x == before);
    match (pos_name, pos_before) {
        (Some(a), Some(b)) => a < b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AAA: Arrange linear deps A->B->C, Act resolve A, Assert order [C, B, A].
    #[test]
    fn test_linear_dependency_order() {
        // Arrange
        let mut r = DependencyResolver::new();
        r.register(RepoPkg::new("C", vec![]));
        r.register(RepoPkg::new("B", vec!["C"]));
        r.register(RepoPkg::new("A", vec!["B"]));
        // Act
        let order = r.resolve("A").unwrap();
        // Assert
        assert!(comes_before(&order, "C", "B"));
        assert!(comes_before(&order, "B", "A"));
    }

    // AAA: Arrange cycle A->B->A, Act resolve, Assert CycleDetected error.
    #[test]
    fn test_cycle_detection() {
        // Arrange
        let mut r = DependencyResolver::new();
        r.register(RepoPkg::new("A", vec!["B"]));
        r.register(RepoPkg::new("B", vec!["A"]));
        // Act
        let result = r.resolve("A");
        // Assert
        assert!(matches!(result, Err(PackageManagerError::CycleDetected(_))));
    }

    // AAA: Arrange resolver without pkg, Act resolve missing, Assert PackageNotFound.
    #[test]
    fn test_missing_package_error() {
        // Arrange
        let r = DependencyResolver::new();
        // Act
        let result = r.resolve("ghost");
        // Assert
        assert!(matches!(result, Err(PackageManagerError::PackageNotFound(_))));
    }

    // AAA: Arrange package with no deps, Act resolve, Assert single-element list.
    #[test]
    fn test_no_deps_single_result() {
        // Arrange
        let mut r = DependencyResolver::new();
        r.register(RepoPkg::new("standalone", vec![]));
        // Act
        let order = r.resolve("standalone").unwrap();
        // Assert
        assert_eq!(order, vec!["standalone".to_string()]);
    }
}
