use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyLevel {
    Normal,   // No emergency
    Elevated, // Increased monitoring
    High,     // Restricted operations
    Critical, // Emergency procedures active
    Lockdown, // Complete lockdown
}

pub struct EmergencyManager {
    current_level: Arc<RwLock<EmergencyLevel>>,
    kill_switch_active: Arc<RwLock<bool>>,
}

impl EmergencyManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_level: Arc::new(RwLock::new(EmergencyLevel::Normal)),
            kill_switch_active: Arc::new(RwLock::new(false)),
        })
    }

    pub fn get_current_level(&self) -> EmergencyLevel {
        self.current_level.read().unwrap().clone()
    }

    pub fn is_kill_switch_active(&self) -> bool {
        *self.kill_switch_active.read().unwrap()
    }

    pub fn can_perform_operation(&self, operation_type: &str) -> bool {
        let level = self.get_current_level();
        let kill_switch = self.is_kill_switch_active();

        if kill_switch {
            return false;
        }

        match level {
            EmergencyLevel::Normal => true,
            EmergencyLevel::Elevated => {
                // Allow most operations with enhanced logging
                !matches!(operation_type, "dangerous" | "experimental")
            }
            EmergencyLevel::High => {
                // Only allow essential operations
                matches!(operation_type, "read" | "validate" | "backup")
            }
            EmergencyLevel::Critical => {
                // Only allow read operations
                operation_type == "read"
            }
            EmergencyLevel::Lockdown => {
                // No operations allowed
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_levels() -> Result<()> {
        let manager = EmergencyManager::new()?;

        assert!(matches!(
            manager.get_current_level(),
            EmergencyLevel::Normal
        ));
        assert!(!manager.is_kill_switch_active());

        Ok(())
    }

    #[test]
    fn test_operation_permissions() -> Result<()> {
        let manager = EmergencyManager::new()?;

        // Normal level should allow all operations
        assert!(manager.can_perform_operation("read"));
        assert!(manager.can_perform_operation("write"));
        assert!(manager.can_perform_operation("dangerous"));

        Ok(())
    }
}
