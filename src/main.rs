use sigmaos_package_manager::{Config, PackageManagerComponent, Component};
fn main() {
    env_logger::init();
    let mut c = PackageManagerComponent::new(Config::default());
    if let Err(e) = c.initialize() { eprintln!("Failed: {}", e); std::process::exit(1); }
    println!("SigmaOS package-manager running");
    let _ = c.shutdown();
}
