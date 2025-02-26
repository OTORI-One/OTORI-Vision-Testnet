#!/bin/bash

# Function to check if validator is running
check_validator() {
    curl -s -X POST -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"is_node_ready","params":[]}' \
    http://localhost:9002/ > /dev/null
    return $?
}

# Clean start if requested
if [ "$1" == "clean" ]; then
    echo "🧹 Cleaning previous validator state..."
    rm -rf .arch_data
fi

# Start the validator
echo "🚀 Starting local validator..."
RUST_LOG=info arch-cli validator start

# Wait for validator to be ready
echo "⏳ Waiting for validator to be ready..."
while ! check_validator; do
    sleep 1
done

echo "✅ Validator is ready!" 