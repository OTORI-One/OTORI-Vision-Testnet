#!/bin/bash
set -e

echo "OTORI Vision Token (OVT) - Complete Deployment Process"
echo "===================================================="

# Check if arch-cli is available in the user's bin directory
if [ ! -f "$HOME/bin/arch-cli" ]; then
    echo "Setting up local arch-cli..."
    mkdir -p ~/bin
    cp /usr/local/bin/arch-cli ~/bin/
    chmod +x ~/bin/arch-cli
    echo "export PATH=~/bin:$PATH" >> ~/.bashrc
    source ~/.bashrc
    echo "✓ Local arch-cli setup complete"
fi

# Set environment variables for Arch Network and Bitcoin testnet
export ARCH_NETWORK="testnet"
export ARCH_BITCOIN_NETWORK="testnet"

# Step 1: Clean deployment preparation
echo ""
echo "Step 1: Preparing clean deployment..."
cd arch-sdk-testnet
chmod +x clean_deploy.sh
./clean_deploy.sh
cd ..
echo "✓ Clean deployment preparation complete"

# Step 2: Deploy the program
echo ""
echo "Step 2: Deploying program to testnet..."
echo "This will upload your program to the Arch Network on Bitcoin testnet."
echo "Running deployment command..."

# Run the deployment command with the user's local arch-cli
~/bin/arch-cli deploy arch-sdk-testnet/build_testnet

# Step 3: Extract the program ID
echo ""
echo "Step 3: Extracting program ID..."
node extract_program_id.js
PROGRAM_ID=$(node -e "
const fs = require('fs');
const keypairPath = './arch-sdk-testnet/build_testnet/otori_program-keypair.json';
const keypairData = fs.readFileSync(keypairPath, 'utf8');
const keypairArray = JSON.parse(keypairData);
const publicKey = keypairArray.slice(keypairArray.length - 32);
const hexString = Buffer.from(publicKey).toString('hex');
console.log(hexString);
")
echo "✓ Program ID extracted: $PROGRAM_ID"

# Step 4: Update frontend configuration
echo ""
echo "Step 4: Updating frontend configuration..."
# Check if .env.local exists in the frontend directory
FRONTEND_DIR="./ovt-fund"
ENV_FILE="$FRONTEND_DIR/.env.local"

if [ ! -d "$FRONTEND_DIR" ]; then
    echo "Error: Frontend directory not found at $FRONTEND_DIR"
    echo "Please specify the correct frontend directory path."
    exit 1
fi

# Create or update the .env.local file
echo "Updating frontend configuration in $ENV_FILE..."

# Create the .env.local file with the program ID
cat > "$ENV_FILE" << EOF
# OTORI Vision Token (OVT) Configuration
# Updated on $(date)

# Program ID for the deployed contract
NEXT_PUBLIC_PROGRAM_ID=$PROGRAM_ID

# Disable mock mode to use the actual deployed contract
NEXT_PUBLIC_MOCK_MODE=false
EOF

echo "✓ Frontend configuration updated successfully!"

# Step 5: Populate initial portfolio positions
echo ""
echo "Step 5: Populating initial portfolio positions..."
npx ts-node scripts/populate-initial-positions.ts
echo "✓ Initial portfolio positions populated"

# Step 6: Mint initial OVT tokens
echo ""
echo "Step 6: Minting initial OVT tokens..."
cd $FRONTEND_DIR
npx ts-node scripts/mint-initial-ovt.ts
cd ..
echo "✓ Initial OVT tokens minted"

# Final instructions
echo ""
echo "===================================================="
echo "DEPLOYMENT COMPLETE!"
echo "===================================================="
echo ""
echo "Program ID: $PROGRAM_ID"
echo ""
echo "You can now start your frontend application with:"
echo "cd $FRONTEND_DIR && npm run dev"
echo ""
echo "The application will be available at http://localhost:3000" 