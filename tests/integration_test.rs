use sigmaos_package_manager::{Config, PackageManagerComponent, PackageManagerService, PackageTransaction, TransactionState, Component};

#[test]
fn test_config_default_aaa() {
    let c = Config::default();
    assert!(c.enabled);
    assert_eq!(c.max_concurrent_installs, 4);
}

#[test]
fn test_transaction_install_commit_aaa() {
    let mut mgr = PackageManagerService::new();
    let tx = mgr.begin_transaction();
    tx.add_install("curl");
    tx.add_install("wget");
    let result = tx.commit();
    assert!(result.is_ok());
    assert_eq!(tx.state, TransactionState::Committed);
}

#[test]
fn test_transaction_empty_commit_fails_aaa() {
    let mut tx = PackageTransaction::new(1);
    assert!(tx.commit().is_err());
}

#[test]
fn test_transaction_rollback_aaa() {
    let mut tx = PackageTransaction::new(1);
    tx.add_remove("old-pkg");
    tx.rollback();
    assert_eq!(tx.state, TransactionState::RolledBack);
}

#[test]
fn test_transaction_op_count_aaa() {
    let mut tx = PackageTransaction::new(42);
    tx.add_install("vim");
    tx.add_install("git");
    tx.add_remove("nano");
    assert_eq!(tx.op_count(), 3);
}

#[test]
fn test_component_lifecycle_aaa() {
    let mut c = PackageManagerComponent::new(Config::default());
    assert!(c.initialize().is_ok());
    assert!(c.health_check().is_ok());
    assert!(c.shutdown().is_ok());
}

#[test]
fn test_component_name_aaa() {
    let c = PackageManagerComponent::new(Config::default());
    assert_eq!(c.name(), "sigmaos-package-manager");
}
