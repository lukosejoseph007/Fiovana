#!/bin/bash
# Comprehensive test runner with CPU monitoring and automatic termination
# This prevents infinite loops and high CPU usage during test compilation

set -e

echo "🔧 Proxemic Safe Test Runner - Preventing Infinite Loops"
echo "========================================================"

# Configuration
MAX_CPU_USAGE=80          # Kill if CPU usage exceeds this for too long
CPU_CHECK_INTERVAL=5      # Check CPU every 5 seconds
MAX_HIGH_CPU_DURATION=30  # Kill if high CPU usage lasts more than 30 seconds
COMPILATION_TIMEOUT=120   # Maximum time allowed for compilation
TEST_TIMEOUT=60           # Maximum time allowed for test execution

# Function to get CPU usage of a process
get_cpu_usage() {
    local pid=$1
    if ps -p "$pid" > /dev/null 2>&1; then
        ps -p "$pid" -o %cpu --no-headers | awk '{print int($1)}'
    else
        echo "0"
    fi
}

# Function to monitor and kill high CPU processes
monitor_and_kill() {
    local pid=$1
    local start_time=$(date +%s)
    local high_cpu_start=0

    while ps -p "$pid" > /dev/null 2>&1; do
        local cpu_usage=$(get_cpu_usage "$pid")
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        if [ "$cpu_usage" -gt "$MAX_CPU_USAGE" ]; then
            if [ "$high_cpu_start" -eq 0 ]; then
                high_cpu_start=$current_time
                echo "⚠️  High CPU usage detected: ${cpu_usage}% (PID: $pid)"
            else
                local high_cpu_duration=$((current_time - high_cpu_start))
                if [ "$high_cpu_duration" -gt "$MAX_HIGH_CPU_DURATION" ]; then
                    echo "🚨 KILLING PROCESS: High CPU usage (${cpu_usage}%) for ${high_cpu_duration}s (PID: $pid)"
                    kill -9 "$pid" 2>/dev/null || true
                    return 1
                fi
            fi
        else
            high_cpu_start=0
        fi

        if [ "$elapsed" -gt "$COMPILATION_TIMEOUT" ]; then
            echo "🚨 KILLING PROCESS: Compilation timeout (${elapsed}s) (PID: $pid)"
            kill -9 "$pid" 2>/dev/null || true
            return 1
        fi

        sleep "$CPU_CHECK_INTERVAL"
    done

    return 0
}

# Function to run tests safely
run_tests_safely() {
    local test_type="$1"
    local test_args="$2"

    echo ""
    echo "🧪 Running $test_type tests with monitoring..."
    echo "   Command: cargo test $test_args"
    echo "   Max CPU: ${MAX_CPU_USAGE}%, Max duration: ${COMPILATION_TIMEOUT}s"

    # Start the test process in background
    timeout "$COMPILATION_TIMEOUT" cargo test $test_args &
    local test_pid=$!

    echo "   Started test process (PID: $test_pid)"

    # Monitor the process
    if monitor_and_kill "$test_pid"; then
        wait "$test_pid"
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            echo "✅ $test_type tests completed successfully"
            return 0
        else
            echo "❌ $test_type tests failed with exit code $exit_code"
            return $exit_code
        fi
    else
        echo "❌ $test_type tests were terminated due to high resource usage"
        return 1
    fi
}

# Function to clean up zombie processes
cleanup_processes() {
    echo "🧹 Cleaning up any zombie processes..."
    pkill -f "cargo.*test" 2>/dev/null || true
    pkill -f "rustc.*test" 2>/dev/null || true
    sleep 2
}

# Main execution
main() {
    echo "🔍 Checking current system state..."

    # Clean up any existing processes
    cleanup_processes

    # Check available memory
    local available_mem=$(free -m | awk 'NR==2{printf "%.0f", $7}')
    echo "   Available memory: ${available_mem}MB"

    if [ "$available_mem" -lt 1000 ]; then
        echo "⚠️  Low memory detected. Consider closing other applications."
    fi

    # Check cargo lock
    if [ -f "target/.rustc_info.json" ]; then
        echo "   Cargo cache found"
    fi

    echo ""
    echo "🚀 Starting safe test execution..."

    # Strategy 1: Try library tests first (fastest, least resource intensive)
    if run_tests_safely "Library" "--lib"; then
        echo "✅ Library tests passed - proceeding with integration tests"
    else
        echo "❌ Library tests failed or were terminated"
        cleanup_processes
        exit 1
    fi

    # Strategy 2: Run integration tests individually with smaller timeout
    local integration_tests=(
        "conflict_detection_tests"
        "event_persistence_tests"
        "file_watcher_tests"
        "health_monitoring_integration_tests"
        "integration_tests"
        "monitoring_tests"
        "notification_tests"
        "workspace_unit_tests"
        "workspace_integration_tests"
        "cross_platform_compatibility_tests"
        "workspace_performance_benchmarks"
        "workspace_security_tests"
        "workspace_edge_case_tests"
    )

    # Strategy 3: Run unit tests that are part of the lib crate
    echo ""
    echo "🔬 Running unit tests..."
    if run_tests_safely "Unit Tests" "--lib -- docx_parser pdf_parser document_commands"; then
        echo "✅ Document parsing unit tests passed"
    else
        echo "❌ Document parsing unit tests failed"
        failed_tests+=("document_parsing_units")
    fi

    echo ""
    echo "🔬 Running integration tests individually..."

    local failed_tests=()
    for test in "${integration_tests[@]}"; do
        if run_tests_safely "Integration ($test)" "--test $test"; then
            echo "✅ $test passed"
        else
            echo "❌ $test failed or was terminated"
            failed_tests+=("$test")
        fi

        # Brief pause between tests to allow system recovery
        sleep 2
    done

    # Summary
    echo ""
    echo "📊 Test Results Summary"
    echo "======================"

    if [ ${#failed_tests[@]} -eq 0 ]; then
        echo "🎉 All tests completed successfully!"
        echo "   Library tests: ✅"
        echo "   Integration tests: ✅ (${#integration_tests[@]}/$(echo ${integration_tests[@]} | wc -w))"
    else
        echo "⚠️  Some tests failed or were terminated:"
        echo "   Library tests: ✅"
        echo "   Failed integration tests: ${failed_tests[*]}"
        echo ""
        echo "💡 This could indicate:"
        echo "   - Resource exhaustion during compilation"
        echo "   - Infinite loops in test code"
        echo "   - Memory leaks in dependencies"
        echo "   - Circular dependencies in the codebase"
    fi

    cleanup_processes

    if [ ${#failed_tests[@]} -eq 0 ]; then
        return 0
    else
        return 1
    fi
}

# Run with CPU monitoring
main "$@"