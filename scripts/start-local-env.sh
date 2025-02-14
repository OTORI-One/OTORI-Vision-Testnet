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
    if nc -zv 127.0.0.1 60401 2>/dev/null; then
        echo "Successfully connected to Electrs RPC"
        return 0
    else
        echo "Failed to connect to Electrs RPC"
        return 1
    fi
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

# Clean up function
cleanup() {
    echo "Cleaning up..."
    bitcoin-cli -regtest stop
    pkill electrs
    pkill arch-cli
    pkill next
    exit 0
}

# Trap Ctrl+C to clean up
trap cleanup INT

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
MAX_ATTEMPTS=60  # Increased from 30 to 60
while ! check_electrs_rpc; do
    ATTEMPTS=$((ATTEMPTS + 1))
    if [ $ATTEMPTS -ge $MAX_ATTEMPTS ]; then
        echo "Electrs RPC failed to start after $MAX_ATTEMPTS attempts"
        echo "Checking Electrs process status..."
        if ps -p $ELECTRS_PID > /dev/null; then
            echo "Electrs process is still running (PID: $ELECTRS_PID)"
            echo "Checking Electrs logs..."
            tail -n 50 ~/Coding/electrs/electrs.log 2>/dev/null || echo "No log file found"
        else
            echo "Electrs process is not running!"
        fi
        exit 1
    fi
    echo "Attempt $ATTEMPTS: Electrs RPC not ready yet, waiting..."
    sleep 2
done

echo "Electrs RPC is ready!"

# Start Arch validator
echo "Starting Arch validator..."
arch-cli validator-start &

# Wait for validator
wait_for_service 8000 "Arch validator"

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