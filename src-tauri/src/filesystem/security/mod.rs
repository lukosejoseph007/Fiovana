pub mod access_control;
pub mod audit_logger;
pub mod file_guard;
pub mod magic_number_validator;
pub mod path_validator;
pub mod permissions;
pub mod permissions_escalation;
pub mod scope;
pub mod scope_validator;
pub mod security_config;

// Public re-exports
#[allow(unused_imports)]
pub use path_validator::PathValidator;
#[allow(unused_imports)]
pub use security_config::SecurityConfig;
