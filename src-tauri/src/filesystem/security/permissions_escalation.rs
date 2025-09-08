use std::time::{SystemTime, UNIX_EPOCH};
use tracing;

/// Represents a user permission escalation request.
pub struct PermissionEscalation {
    pub user_approved: bool,
    #[allow(dead_code)]
    expiration_time: u64, // used internally, warning suppressed
}

impl PermissionEscalation {
    /// Creates a permission escalation result from a manual approval input.
    #[allow(dead_code)]
    pub fn from_user_input(user_approved: bool) -> Self {
        let expiration_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60; // Permission expires after 60 seconds

        Self::log_escalation_event(user_approved);

        Self {
            user_approved,
            expiration_time,
        }
    }

    /// Checks if the permission is still valid.
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        if !self.user_approved {
            return false;
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        current_time <= self.expiration_time
    }

    #[allow(dead_code)]
    fn log_escalation_event(approved: bool) {
        tracing::info!("Escalation event: Approved - {}", approved);
    }
}
