#!/bin/bash
# Safe test runner with timeouts to prevent infinite loops

echo "Running Rust tests in batches with timeouts..."

# Run library tests first
echo "=== Running library tests ==="
timeout 60s cargo test --lib
LIB_RESULT=$?
if [ $LIB_RESULT -eq 124 ]; then
    echo "❌ Library tests timed out!"
    exit 1
elif [ $LIB_RESULT -ne 0 ]; then
    echo "❌ Library tests failed with exit code $LIB_RESULT!"
    exit 1
else
    echo "✅ Library tests passed!"
fi

# Run integration tests individually with timeouts
echo ""
echo "=== Running integration tests individually ==="

INTEGRATION_TESTS=(
    "conflict_detection_tests"
    "event_persistence_tests"
    "file_watcher_tests"
    "health_monitoring_integration_tests"
    "integration_tests"
    "monitoring_tests"
    "notification_tests"
)

for test in "${INTEGRATION_TESTS[@]}"; do
    echo "Running $test..."
    timeout 60s cargo test --test "$test"
    if [ $? -eq 124 ]; then
        echo "❌ $test timed out!"
        exit 1
    elif [ $? -ne 0 ]; then
        echo "❌ $test failed!"
        exit 1
    else
        echo "✅ $test passed!"
    fi
done

echo ""
echo "🎉 All tests completed successfully!"