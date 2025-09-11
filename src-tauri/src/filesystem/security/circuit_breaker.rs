use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject calls
    HalfOpen, // Testing if service recovered
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub success_threshold: u32, // For half-open state
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    next_attempt: Option<Instant>,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            next_attempt: None,
        }
    }
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
    name: String,
}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::default())),
            name,
        }
    }

    pub fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Check if we should allow the call
        if !self.should_allow_call() {
            return Err(anyhow!("Circuit breaker '{}' is OPEN", self.name));
        }

        // Execute the operation
        let start_time = Instant::now();
        let result = operation();

        match result {
            Ok(success) => {
                self.record_success();
                log::debug!(
                    "Circuit breaker '{}' call succeeded in {:?}",
                    self.name,
                    start_time.elapsed()
                );
                Ok(success)
            }
            Err(e) => {
                self.record_failure();
                log::warn!("Circuit breaker '{}' call failed: {}", self.name, e);
                Err(e)
            }
        }
    }

    fn should_allow_call(&self) -> bool {
        let state = self.state.read().unwrap();

        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = state.last_failure_time {
                    if Instant::now().duration_since(last_failure) >= self.config.recovery_timeout {
                        drop(state); // Release read lock
                        self.transition_to_half_open();
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&self) {
        let mut state = self.state.write().unwrap();

        match state.state {
            CircuitState::Closed => {
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    log::info!(
                        "Circuit breaker '{}' recovered, transitioning to CLOSED",
                        self.name
                    );
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_failure_time = None;
                }
            }
            CircuitState::Open => {
                // This shouldn't happen, but handle gracefully
                log::warn!(
                    "Circuit breaker '{}' recorded success while OPEN",
                    self.name
                );
            }
        }
    }

    fn record_failure(&self) {
        let mut state = self.state.write().unwrap();

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    log::error!(
                        "Circuit breaker '{}' opening due to {} failures",
                        self.name,
                        state.failure_count
                    );
                    state.state = CircuitState::Open;
                    state.next_attempt = Some(Instant::now() + self.config.recovery_timeout);
                }
            }
            CircuitState::HalfOpen => {
                log::warn!(
                    "Circuit breaker '{}' failed in half-open state, reopening",
                    self.name
                );
                state.state = CircuitState::Open;
                state.success_count = 0;
                state.next_attempt = Some(Instant::now() + self.config.recovery_timeout);
            }
            CircuitState::Open => {
                // Already open, just update timing
                state.next_attempt = Some(Instant::now() + self.config.recovery_timeout);
            }
        }
    }

    fn transition_to_half_open(&self) {
        let mut state = self.state.write().unwrap();
        if state.state == CircuitState::Open {
            log::info!("Circuit breaker '{}' transitioning to HALF_OPEN", self.name);
            state.state = CircuitState::HalfOpen;
            state.success_count = 0;
        }
    }
}

// Circuit breaker manager for the entire application
pub struct CircuitBreakerManager {
    breakers: Arc<Mutex<HashMap<String, Arc<CircuitBreaker>>>>,
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self {
            breakers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create(
        &self,
        name: &str,
        config: Option<CircuitBreakerConfig>,
    ) -> Arc<CircuitBreaker> {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get(name) {
            return breaker.clone();
        }

        let breaker = Arc::new(CircuitBreaker::new(
            name.to_string(),
            config.unwrap_or_default(),
        ));

        breakers.insert(name.to_string(), breaker.clone());
        breaker
    }
}

// Enhanced file operations with circuit breaker protection
// Note: FileOperation implementation commented out due to missing dependency
// use crate::filesystem::operations::FileOperation;

// impl FileOperation {
//     pub async fn read_file_with_circuit_breaker(
//         path: &std::path::Path,
//         breaker_manager: &CircuitBreakerManager,
//     ) -> Result<String> {
//         let breaker = breaker_manager.get_or_create(
//             "file_read",
//             Some(CircuitBreakerConfig {
//                 failure_threshold: 3,
//                 recovery_timeout: Duration::from_secs(30),
//                 call_timeout: Duration::from_secs(10),
//                 success_threshold: 2,
//             }),
//         );
//
//         breaker
//             .call(|| {
//                 std::fs::read_to_string(path)
//                     .map_err(|e| anyhow!("Failed to read file {:?}: {}", path, e))
//             })
//             .await
//     }
//
//     pub async fn validate_file_with_circuit_breaker(
//         path: &std::path::Path,
//         breaker_manager: &CircuitBreakerManager,
//     ) -> Result<bool> {
//         let breaker = breaker_manager.get_or_create(
//             "file_validation",
//             Some(CircuitBreakerConfig {
//                 failure_threshold: 5,
//                 recovery_timeout: Duration::from_secs(60),
//                 call_timeout: Duration::from_secs(15),
//                 success_threshold: 3,
//             }),
//         );
//
//         breaker.call(|| {
//             // Use your existing validation logic
//             crate::filesystem::security::magic_number_validator::MagicNumberValidator::new()
//                 .validate_file(path)
//                 .map(|result| matches!(result, crate::filesystem::security::magic_number_validator::ValidationResult::Approved(_)))
//         }).await
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_operation() {
        let breaker = CircuitBreaker::new(
            "test".to_string(),
            CircuitBreakerConfig {
                failure_threshold: 2,
                recovery_timeout: Duration::from_secs(1),
                success_threshold: 1,
            },
        );

        // First call should succeed
        let result = breaker.call(|| Ok::<String, anyhow::Error>("success".to_string()));
        assert!(result.is_ok());

        // Fail twice to trigger circuit breaker
        for _ in 0..2 {
            let result = breaker.call(|| Err::<String, anyhow::Error>(anyhow!("test failure")));
            assert!(result.is_err());
        }

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_secs(2));

        // Should be able to call again after timeout
        let result = breaker.call(|| Ok::<String, anyhow::Error>("recovered".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new();

        let breaker1 = manager.get_or_create("test1", None);
        let breaker2 = manager.get_or_create("test2", None);

        // Test that breakers are created successfully
        let result1 = breaker1.call(|| Ok::<String, anyhow::Error>("test1".to_string()));
        let result2 = breaker2.call(|| Ok::<String, anyhow::Error>("test2".to_string()));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
