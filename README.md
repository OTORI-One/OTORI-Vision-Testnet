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
- PowerShell
- Access to Bitcoin network (testnet for development)
- Arch Network CLI tools

## Development Approaches

### Option 1: Frontend with Mocked Backend (Recommended for now)

1. Frontend Setup:
```powershell
git clone <repository-url>
cd ovt-fund
npm install
```

2. Configure mock environment:
```powershell
# Copy the example env file
cp .env.local.example .env.local

# Update with mock values for development
NEXT_PUBLIC_PROGRAM_ID=mock_program_id
NEXT_PUBLIC_TREASURY_ADDRESS=mock_treasury_address
NEXT_PUBLIC_ARCH_ENDPOINT=http://localhost:8000
NEXT_PUBLIC_MOCK_MODE=true
```

3. Run the development server:
```powershell
npm run dev
```

4. Open [http://localhost:3000](http://localhost:3000) in your browser.

The frontend will use mock data for:
- NAV calculations
- Portfolio values
- Bitcoin transactions
- Wallet interactions

### Option 2: Full Stack Development (Pending Arch Network Tools)

**Note:** Full backend integration is pending availability of Arch Network development tools. We'll update these instructions once the tools are ready for public use.

Requirements:
- Rust and Cargo
- Arch Network CLI tools (coming soon)
- Bitcoin testnet access
- Hardware wallet for treasury management

## Getting Started

### Frontend Setup

1. Clone the repository:
```powershell
git clone <repository-url>
cd ovt-fund
```

2. Install dependencies:
```powershell
npm install
```

3. Configure environment:
   - Copy `.env.local.example` to `.env.local`
   - Update the following values:
     - `NEXT_PUBLIC_PROGRAM_ID`: Obtained after deploying your program to Arch Network
     - `NEXT_PUBLIC_TREASURY_ADDRESS`: Your Bitcoin treasury address (where funds will be managed)
     - `NEXT_PUBLIC_ARCH_ENDPOINT`: Your Arch Network node endpoint (use localhost:8000 for development)

4. Run the development server:
```powershell
npm run dev
```

5. Open [http://localhost:3000](http://localhost:3000) in your browser.

### Rust Setup (for OVT Program)

1. Install Rust using PowerShell (run as Administrator):
```powershell
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
# Follow the installation prompts, then restart your PowerShell
```

2. Verify Rust installation:
```powershell
rustc --version
cargo --version
```

3. Clone and build Arch Network tools:
```powershell
# Create a separate directory for Arch tools
mkdir C:\arch-tools
cd C:\arch-tools

# Clone the required repositories
git clone https://github.com/Arch-Network/arch-cli.git
git clone https://github.com/Arch-Network/arch-node.git

# Build arch-cli
cd arch-cli
cargo build --release
cd ..

# Build local validator from arch-node
cd arch-node
cargo build --release
cd ..

# Add the binary directories to your PATH
$env:PATH += ";C:\arch-tools\arch-cli\target\release;C:\arch-tools\arch-node\target\release"

# Return to your project directory
cd C:\Users\admin\Documents\Coding\OVT_on_arch
```

4. Verify installation:
```powershell
arch-cli --version
arch-local-validator --version
```

### Backend Setup

#### Development Environment
1. Start the local Arch Network validator:
```powershell
# Make sure you're in the arch-node directory
cd arch-node
cargo run --bin arch-local-validator
```

2. Deploy the OVT program locally:
```powershell
cd ovt-program
arch-cli program deploy --local
# Note the program ID output - you'll need this for .env.local
```

3. Generate a test treasury address:
```powershell
arch-cli address generate --network testnet
# Use this address as your NEXT_PUBLIC_TREASURY_ADDRESS in .env.local
```

#### Production Environment
1. Deploy to Arch Network testnet:
```powershell
arch-cli program deploy --network testnet
# Note the program ID - update this in your production .env
```

2. Set up a secure Bitcoin treasury address:
   - Generate using a hardware wallet
   - Ensure proper key management and backup
   - Consider multi-sig setup for additional security

3. Run an Arch Network node:
   - Set up a dedicated server
   - Install Arch Network node software
   - Configure Bitcoin node connection
   - Set up monitoring and maintenance

### Testing

1. Run the OVT program tests:
```powershell
cd ovt-program
cargo test -- --nocapture
```

2. Test specific flows:
```powershell
cargo test test_full_flow -- --nocapture
```

## Project Structure

```
ovt-fund/
├── components/          # Reusable React components
├── pages/              # Next.js pages
├── public/             # Static assets
├── src/
│   ├── hooks/          # Custom React hooks
│   └── lib/            # Utility functions and clients
├── styles/             # CSS styles
└── types/              # TypeScript type definitions

ovt-program/
├── src/
│   ├── lib.rs          # Main program logic
│   ├── client.rs       # Arch Network program client
│   └── runes_client.rs # Bitcoin Runes protocol client
└── tests/              # Integration tests
```

## Development Workflow

1. Start with local development:
   - Run local Arch validator
   - Deploy program locally
   - Test with mock Bitcoin transactions

2. Move to testnet:
   - Deploy to Arch testnet
   - Test with real Bitcoin testnet transactions
   - Verify all flows with test funds

3. Production deployment:
   - Deploy to Arch mainnet
   - Configure secure treasury management
   - Enable real Bitcoin transactions

## Security Considerations

- Always test thoroughly on testnet first
- Use hardware wallets for treasury management
- Implement proper monitoring and alerts
- Regular security audits
- Backup and recovery procedures

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 