#!/bin/bash

# CPU Monitoring Script for Document Sync
# This script monitors CPU usage and automatically kills the process if it gets stuck

echo "🔍 Starting CPU monitoring for document sync..."
echo "Press Ctrl+C to stop monitoring"

# Function to get CPU usage for Proxemic processes
get_cpu_usage() {
    local cpu_usage=$(ps aux | grep -E "(proxemic|tauri)" | grep -v grep | awk '{sum += $3} END {printf "%.1f", sum}')
    echo "${cpu_usage:-0.0}"
}

# Function to kill high CPU processes
kill_high_cpu_processes() {
    echo "🔴 EMERGENCY: Killing high CPU processes to prevent system hang"
    pkill -f "proxemic" 2>/dev/null || true
    pkill -f "tauri" 2>/dev/null || true
    echo "✅ Processes terminated"
}

# Monitor CPU usage every 5 seconds
start_time=$(date +%s)
high_cpu_count=0
max_runtime=300  # 5 minutes max runtime

while true; do
    current_time=$(date +%s)
    runtime=$((current_time - start_time))

    # Safety: Kill after max runtime to prevent infinite hanging
    if [ $runtime -gt $max_runtime ]; then
        echo "⏰ TIMEOUT: Maximum runtime (${max_runtime}s) exceeded"
        kill_high_cpu_processes
        break
    fi

    cpu_usage=$(get_cpu_usage)
    echo "$(date '+%H:%M:%S') - CPU Usage: ${cpu_usage}% (Runtime: ${runtime}s)"

    # Check if CPU usage is critically high
    if (( $(echo "$cpu_usage > 80.0" | bc -l) )); then
        high_cpu_count=$((high_cpu_count + 1))
        echo "⚠️  WARNING: High CPU usage detected (${cpu_usage}%) - Count: $high_cpu_count"

        # If CPU usage is high for too long, kill processes
        if [ $high_cpu_count -ge 3 ]; then
            echo "🔴 CRITICAL: Sustained high CPU usage - terminating processes"
            kill_high_cpu_processes
            break
        fi
    else
        high_cpu_count=0
    fi

    sleep 5
done

echo "🏁 CPU monitoring completed"