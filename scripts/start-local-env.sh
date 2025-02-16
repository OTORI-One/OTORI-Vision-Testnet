#!/bin/bash

# Function to check if Bitcoin Core is responsive
check_bitcoin() {
    bitcoin-cli -regtest getblockchaininfo >/dev/null 2>&1
    return $?
}

# Function to check if a port is in use (for other services)
check_port() {
    lsof -i:$1 > /dev/null 2>&1
    return $?
}

# Function to check if Electrs REST API is responsive
check_electrs() {
    curl -s http://127.0.0.1:3002/blocks/tip/hash >/dev/null 2>&1
    return $?
}

# Function to check if Electrs RPC is responsive
check_electrs_rpc() {
    echo "Attempting to connect to Electrs RPC on 127.0.0.1:60401..."
    if timeout 2 bash -c "echo > /dev/tcp/127.0.0.1/60401" 2>/dev/null; then
        echo "Successfully connected to Electrs RPC"
        return 0
    else
        echo "Failed to connect to Electrs RPC"
        if ss -tlnp 2>/dev/null | grep -q ":60401"; then
            echo "Port 60401 is being listened to, but connection failed"
            ss -tlnp 2>/dev/null | grep ":60401"
        else
            echo "No process is listening on port 60401"
        fi
        return 1
    fi
}

# Function to check if Arch validator is responsive
check_validator() {
    # First check if the port is open
    if ! check_port 8000; then
        return 1
    fi
    
    # Then check if the validator is actually responding
    if ! curl -s http://127.0.0.1:8000/health >/dev/null 2>&1; then
        return 1
    fi
    
    return 0
}

# Function to wait for Bitcoin Core
wait_for_bitcoin() {
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for Bitcoin Core to be ready..."
    while ! check_bitcoin; do
        if [ $attempt -ge $max_attempts ]; then
            echo "Bitcoin Core failed to start after $max_attempts attempts"
            exit 1
        fi
        echo "Attempt $attempt: Bitcoin Core not ready yet..."
        sleep 2
        ((attempt++))
    done
    echo "Bitcoin Core is ready!"
}

# Function to wait for other services
wait_for_service() {
    local port=$1
    local service=$2
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for $service to be ready..."
    while ! check_port $port; do
        if [ $attempt -ge $max_attempts ]; then
            echo "$service failed to start after $max_attempts attempts"
            exit 1
        fi
        echo "Attempt $attempt: $service not ready yet..."
        sleep 2
        ((attempt++))
    done
    echo "$service is ready!"
}

# Function to wait for Electrs
wait_for_electrs() {
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for Electrs to be ready..."
    while ! check_electrs; do
        if [ $attempt -ge $max_attempts ]; then
            echo "Electrs failed to start after $max_attempts attempts"
            exit 1
        fi
        echo "Attempt $attempt: Electrs not ready yet..."
        sleep 2
        ((attempt++))
    done
    echo "Electrs is ready!"
}

# Function to wait for Electrs RPC
wait_for_electrs_rpc() {
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for Electrs RPC to be ready..."
    while ! check_electrs_rpc; do
        if [ $attempt -ge $max_attempts ]; then
            echo "Electrs RPC failed to start after $max_attempts attempts"
            exit 1
        fi
        echo "Attempt $attempt: Electrs RPC not ready yet..."
        sleep 2
        ((attempt++))
    done
    echo "Electrs RPC is ready!"
}

# Function to wait for Arch validator
wait_for_validator() {
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for Arch validator to be ready..."
    
    # First ensure the validator directory exists and is clean
    echo "Preparing validator directory..."
    rm -rf .arch-validator/*
    mkdir -p .arch-validator
    
    # Start the validator
    echo "Starting Arch validator..."
    RUST_LOG=info arch-cli validator-start > .arch-validator/validator.log 2>&1 &
    VALIDATOR_PID=$!
    
    # Wait for it to be ready
    while ! check_validator; do
        if [ $attempt -ge $max_attempts ]; then
            echo "Arch validator failed to start after $max_attempts attempts"
            echo "Checking validator process status..."
            if ps -p $VALIDATOR_PID > /dev/null; then
                echo "Validator process is still running (PID: $VALIDATOR_PID)"
                echo "Last few lines of validator log:"
                tail -n 20 .arch-validator/validator.log
            else
                echo "Validator process is not running!"
                echo "Last few lines of validator log:"
                tail -n 20 .arch-validator/validator.log
            fi
            exit 1
        fi
        echo "Attempt $attempt: Arch validator not ready yet..."
        sleep 2
        ((attempt++))
    done
    echo "Arch validator is ready!"
}

# Clean up function
cleanup() {
    echo "Cleaning up..."
    echo "Stopping Bitcoin Core..."
    bitcoin-cli -regtest stop 2>/dev/null || true
    
    echo "Stopping Electrs processes..."
    pkill electrs || true
    # Wait for processes to stop
    sleep 2
    # Force kill any remaining Electrs processes
    pkill -9 electrs 2>/dev/null || true
    
    echo "Stopping other services..."
    pkill arch-cli || true
    pkill next || true
    exit 0
}

# Initial cleanup to ensure clean state
initial_cleanup() {
    echo "Performing initial cleanup..."
    
    echo "Stopping any running Bitcoin Core..."
    bitcoin-cli -regtest stop 2>/dev/null || true
    sleep 2
    
    echo "Stopping any running Electrs processes..."
    pkill electrs || true
    sleep 2
    pkill -9 electrs 2>/dev/null || true
    
    # Check if port 60401 is still in use
    if ss -tlnp 2>/dev/null | grep -q ":60401"; then
        echo "Warning: Port 60401 is still in use by:"
        ss -tlnp | grep ":60401"
        echo "Attempting to free up port 60401..."
        fuser -k 60401/tcp 2>/dev/null || true
        sleep 2
    fi
}

# Trap Ctrl+C to clean up
trap cleanup INT

# Perform initial cleanup
initial_cleanup

# Create necessary directories
mkdir -p .arch-validator
mkdir -p "$HOME/.bitcoin"

# Clean up any existing Bitcoin Core data
echo "Cleaning up existing Bitcoin Core data..."
bitcoin-cli -regtest stop 2>/dev/null || true
sleep 2
rm -rf "$HOME/.bitcoin/regtest"
mkdir -p "$HOME/.bitcoin/regtest"

# Clean up Electrs database
echo "Cleaning up Electrs database..."
cd ~/Coding/electrs && rm -rf ./db
mkdir -p ./db

# Start Bitcoin Core in regtest mode
echo "Starting Bitcoin Core in regtest mode..."
bitcoind -regtest -daemon \
    -datadir="$HOME/.bitcoin" \
    -rpcuser=bitcoin \
    -rpcpassword=bitcoinpass \
    -rpcallowip=0.0.0.0/0 \
    -rpcbind=0.0.0.0 \
    -server=1 \
    -txindex=1 \
    -fallbackfee=0.001

# Wait for Bitcoin Core
wait_for_bitcoin

# Start Electrs
echo "Starting Electrs..."
cd ~/Coding/electrs && \
cargo run --release --bin electrs -- \
    -vvvv \
    --daemon-dir ~/.bitcoin \
    --network regtest \
    --cookie bitcoin:bitcoinpass \
    --main-loop-delay 0 \
    --monitoring-addr "0.0.0.0:0" \
    --electrum-rpc-addr "127.0.0.1:60401" \
    --http-addr "127.0.0.1:3002" &

# Store the Electrs PID
ELECTRS_PID=$!
echo "Electrs started with PID: $ELECTRS_PID"

cd - > /dev/null  # Return to previous directory

# Give Electrs a moment to initialize
sleep 5

# Wait for Electrs REST API
wait_for_electrs

# Wait for Electrs RPC with increased timeout
echo "Waiting for Electrs RPC with increased timeout..."
ATTEMPTS=0
MAX_ATTEMPTS=30  # Reduced back to 30 as we have better diagnostics now
while ! check_electrs_rpc; do
    ATTEMPTS=$((ATTEMPTS + 1))
    if [ $ATTEMPTS -ge $MAX_ATTEMPTS ]; then
        echo "Electrs RPC failed to start after $MAX_ATTEMPTS attempts"
        echo "Checking Electrs process status..."
        if ps -p $ELECTRS_PID > /dev/null; then
            echo "Electrs process is still running (PID: $ELECTRS_PID)"
            echo "Checking Electrs logs..."
            tail -n 50 ~/Coding/electrs/electrs.log 2>/dev/null || echo "No log file found"
            echo "Checking port status..."
            ss -tlnp 2>/dev/null | grep ":60401" || echo "Port 60401 is not being listened to"
        else
            echo "Electrs process is not running!"
        fi
        exit 1
    fi
    echo "Attempt $ATTEMPTS: Electrs RPC not ready yet, waiting..."
    sleep 2
done

echo "Electrs RPC is ready!"

# Start Arch validator and wait for it
wait_for_validator

# Start frontend development server
echo "Starting Next.js development server..."
cd ovt-fund && npm run dev &

# Wait for Next.js
wait_for_service 3000 "Next.js"

echo "All services are running!"
echo "Frontend: http://localhost:3000"
echo "Arch validator: http://localhost:8000"
echo "Electrs: http://localhost:3002"
echo "Bitcoin RPC: http://localhost:18443"

# Keep script running
while true; do sleep 1; done 