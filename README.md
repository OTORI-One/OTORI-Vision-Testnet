# OTORI Vision Token (OVT) Dashboard

A Next.js application for managing and tracking the OTORI Vision Token (OVT), a transparent and efficient on-chain VC fund.

## Features

- Real-time NAV (Net Asset Value) tracking and visualization
- Bitcoin wallet integration for buying and selling OVT
- Portfolio performance monitoring
- Transparent fund management

## Prerequisites

- Node.js 16.x or later
- npm 7.x or later
- A Bitcoin wallet (e.g., Xverse, Unisat, or Leather)
- Rust and Cargo
- Linux environment (tested on Manjaro)
- Access to Bitcoin network (regtest for development)
- Arch Network CLI tools

## Development Environment Setup

### 1. Environment Configuration
```bash
# Copy the example environment file
cp ovt-fund/.env.example ovt-fund/.env.local

# Edit .env.local with your specific configuration
# For local development, the default values in .env.example should work
```

### 2. System Dependencies
```bash
# For Manjaro/Arch Linux
sudo pacman -S base-devel clang llvm rocksdb snappy zlib bzip2
```

### 3. Bitcoin Core Setup
```bash
# Install Bitcoin Core
sudo pacman -S bitcoin

# Create Bitcoin configuration
mkdir -p ~/.bitcoin
echo "regtest=1
server=1
rpcuser=bitcoin
rpcpassword=bitcoinpass
rpcallowip=0.0.0.0/0
rpcbind=0.0.0.0" > ~/.bitcoin/bitcoin.conf

# Start Bitcoin daemon in regtest mode
bitcoind -regtest -daemon
```

### 4. Electrs Setup
```bash
# Create directory for Arch tools
mkdir -p ~/Coding/arch-tools
cd ~/Coding/arch-tools

# Clone and build Electrs
git clone https://github.com/Arch-Network/electrs.git
cd electrs
cargo build --release
```

### 5. Arch Validator Setup
```bash
# Create project directory
mkdir -p ~/Coding/OTORI-Vision-Testnet
cd ~/Coding/OTORI-Vision-Testnet

# Initialize validator directory
mkdir -p .arch-validator
```

## Running the Development Environment

1. Start Bitcoin Core (if not already running):
```bash
bitcoind -regtest -daemon -rpcuser=bitcoin -rpcpassword=bitcoinpass -rpcallowip=0.0.0.0/0 -rpcbind=0.0.0.0 -server=1
```

2. Run Electrs (in a separate terminal):
```bash
cd ~/Coding/arch-tools/electrs
cargo run --release --bin electrs -- -vvvv --daemon-dir ~/.bitcoin --network regtest --cookie bitcoin:bitcoinpass --main-loop-delay 0
```

3. Start Arch validator (in another terminal):
```bash
cd ~/Coding/OTORI-Vision-Testnet
arch-cli validator-start
```

4. Start the frontend development server:
```bash
npm run dev
```

## Development Approaches

### Option 1: Frontend with Mocked Backend

1. Configure mock environment:
```bash
# Copy the example env file
cp .env.local.example .env.local

# Update with mock values for development
NEXT_PUBLIC_PROGRAM_ID=mock_program_id
NEXT_PUBLIC_TREASURY_ADDRESS=mock_treasury_address
NEXT_PUBLIC_ARCH_ENDPOINT=http://localhost:8000
NEXT_PUBLIC_MOCK_MODE=true
```

2. Initialize mock data:
```bash
# Populate initial portfolio positions
npm run populate-positions

# Mint initial OVT tokens
npm run mint-ovt
```

3. Run the development server:
```bash
npm run dev
```

Note: Mock data will be stored in `ovt-fund/src/mock-data/`. This data should not be committed to the repository and will be removed before mainnet deployment.

### Option 2: Full Stack Development

Requirements:
- Running Bitcoin Core in regtest mode
- Running Electrs indexer
- Running Arch validator
- Hardware wallet for treasury management (production only)

## Project Structure

```
ovt-fund/               # Next.js frontend application
├── components/         # React components
│   ├── admin/         # Admin dashboard components
│   └── __tests__/     # Component tests
├── pages/             # Next.js pages
├── public/            # Static assets
├── src/
│   ├── hooks/         # Custom React hooks
│   ├── lib/           # Utility functions and clients
│   ├── mock-data/     # Development mock data (not for production)
│   └── utils/         # Helper functions
├── styles/            # CSS styles
└── types/             # TypeScript type definitions

ovt-program/           # Rust program for Arch Network
├── src/
│   ├── lib.rs         # Main program logic
│   ├── client.rs      # Arch Network program client
│   ├── runes_client.rs # Bitcoin Runes protocol client
│   └── error.rs       # Error definitions
└── tests/             # Integration tests

docs/                  # Project documentation
└── BACKLOG.md         # Development backlog
```

## Stopping the Development Environment

1. Stop Bitcoin Core:
```bash
bitcoin-cli -regtest stop
```

2. Stop Electrs and Arch validator:
- Use Ctrl+C in their respective terminals

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 