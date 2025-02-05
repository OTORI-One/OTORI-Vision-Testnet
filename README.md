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
- A Bitcoin wallet (e.g., Xverse)
- Rust and Cargo
- PowerShell

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

3. Run the development server:
```powershell
npm run dev
```

4. Open [http://localhost:3000](http://localhost:3000) in your browser.

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

3. Install Arch Network's local validator:
```powershell
cargo install arch-local-validator
```

### Local Testing

1. Run the OVT program tests:
```powershell
cd ovt-program
cargo test -- --nocapture
```

2. Run specific test:
```powershell
cargo test test_full_flow -- --nocapture
```

## Project Structure

```
ovt-fund/
├── components/          # Reusable React components
├── pages/              # Next.js pages
├── public/             # Static assets
├── src/                # Source files
│   └── hooks/          # Custom React hooks
├── styles/             # CSS styles
└── types/              # TypeScript type definitions
```

## Development

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm start` - Start production server
- `npm run lint` - Run ESLint

### Technology Stack

- Next.js - React framework
- TypeScript - Type safety
- Tailwind CSS - Styling
- Recharts - Data visualization
- Bitcoin wallet integration
- Rust - Smart contract development
- Arch Network - Bitcoin L2 platform

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 