# Claude Rules

## Testing

**CRITICAL: Cargo Test Safety Rule**

- NEVER run `cargo test` in any case as it might cause infinite loops and system crashes
- Always use `./prevent_test_loops.sh` as it is the correct testing approach for this project to prevent infinite loops
- If you create new tests, ensure you add them to the `prevent_test_loops.sh` script so it stays up to date and will properly execute these tests with timeout protection to prevent infinite loops