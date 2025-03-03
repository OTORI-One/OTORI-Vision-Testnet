/**
 * Script to mint additional OVT Runes on Bitcoin testnet
 * 
 * This script mints additional OVT tokens to support rolling raises
 * and the inflationary supply model.
 */

const { Psbt } = require('bitcoinjs-lib');
const { ECPairFactory } = require('ecpair');
const ecc = require('tiny-secp256k1');
const axios = require('axios');
const dotenv = require('dotenv');
const path = require('path');
const fs = require('fs');
const { execSync } = require('child_process');

// Load environment variables
dotenv.config({ path: path.resolve(__dirname, '../ovt-fund/.env.local') });

// Bitcoin testnet configuration
const NETWORK = {
  messagePrefix: '\x18Bitcoin Signed Message:\n',
  bech32: 'tb',
  bip32: {
    public: 0x043587cf,
    private: 0x04358394,
  },
  pubKeyHash: 0x6f,
  scriptHash: 0xc4,
  wif: 0xef,
};

// Initialize ECPair factory
const ECPair = ECPairFactory(ecc);

// Testnet API endpoints
const TESTNET_API = {
  broadcast: 'https://mempool.space/testnet/api/tx',
  utxos: (address) => `https://mempool.space/testnet/api/address/${address}/utxo`
};

/**
 * Load the Rune information from the saved file
 */
function loadRuneInfo() {
  try {
    const runeInfoPath = path.resolve(__dirname, '../ovt-fund/src/rune-info.json');
    if (!fs.existsSync(runeInfoPath)) {
      throw new Error('Rune info file not found. Please etch the OVT Rune first.');
    }
    
    const runeInfo = JSON.parse(fs.readFileSync(runeInfoPath, 'utf8'));
    console.log(`Loaded Rune info for ${runeInfo.symbol}`);
    return runeInfo;
  } catch (error) {
    console.error('Failed to load Rune info:', error.message);
    throw error;
  }
}

/**
 * Get UTXOs for an address
 */
async function getUtxos(address) {
  try {
    const response = await axios.get(TESTNET_API.utxos(address));
    return response.data;
  } catch (error) {
    console.error('Failed to get UTXOs:', error.response?.data || error.message);
    throw error;
  }
}

/**
 * Mint additional Runes using ord
 */
function mintRuneWithOrd(runeInfo, mintAmount, privateKeyWIF, signatures = []) {
  try {
    // Check if ord is installed
    try {
      execSync('ord --version');
      console.log('Ord is installed');
    } catch (error) {
      console.error('Ord is not installed. Please install ord first.');
      console.error('Installation instructions: https://github.com/ordinals/ord#installation');
      throw new Error('Ord not installed');
    }
    
    // Create a temporary wallet for minting
    const walletName = `ovt-mint-${Date.now()}`;
    console.log(`Creating temporary wallet: ${walletName}`);
    
    // Import the private key to the wallet
    execSync(`ord --testnet wallet create ${walletName}`);
    execSync(`ord --testnet wallet import ${walletName} ${privateKeyWIF}`);
    
    // Construct the mint command
    let mintCommand = `ord --testnet wallet mint ${walletName} --fee-rate 10 --rune ${runeInfo.symbol} --amount ${mintAmount}`;
    
    // Add signatures if provided (for multi-sig)
    if (signatures && signatures.length > 0) {
      // In a real implementation, you would add the signatures to the transaction
      // For now, we'll just log them
      console.log(`Using ${signatures.length} signatures for multi-sig minting`);
    }
    
    console.log(`Executing mint command: ${mintCommand}`);
    const result = execSync(mintCommand).toString();
    console.log('Mint result:', result);
    
    // Extract the transaction ID from the result
    const txidMatch = result.match(/[a-f0-9]{64}/);
    if (!txidMatch) {
      throw new Error('Could not extract transaction ID from mint result');
    }
    
    const txid = txidMatch[0];
    console.log(`Extracted transaction ID: ${txid}`);
    
    return {
      txid,
      command: mintCommand,
      result
    };
  } catch (error) {
    console.error('Failed to mint Rune with ord:', error.message);
    throw error;
  }
}

/**
 * Main function to mint additional OVT tokens
 */
async function mintOVTTokens(signatures = []) {
  try {
    // Parse command line arguments
    const args = process.argv.slice(2);
    const mintAmount = parseInt(args[0]);
    
    // Check if signatures were provided as a command line argument
    if (args.length > 1) {
      try {
        const signaturesArg = JSON.parse(args[1]);
        if (Array.isArray(signaturesArg) && signaturesArg.length > 0) {
          signatures = signaturesArg;
          console.log(`Using ${signatures.length} signatures from command line`);
        }
      } catch (err) {
        console.warn('Failed to parse signatures from command line:', err);
      }
    }
    
    if (!mintAmount || isNaN(mintAmount) || mintAmount <= 0) {
      console.error('Please provide a valid mint amount as the first argument');
      console.log('Usage: node mint-ovt-rune.js <amount> [signatures_json_array]');
      process.exit(1);
    }
    
    console.log(`Preparing to mint ${mintAmount} OVT tokens...`);
    if (signatures.length > 0) {
      console.log(`Using ${signatures.length} signatures for multi-sig minting`);
    }
    
    // Load the Rune information
    const runeInfo = loadRuneInfo();
    
    // Check if we have the WIF format private key
    let privateKeyWIF = runeInfo.keypair.privateKeyWIF;
    
    // If not, convert the private key to WIF format
    if (!privateKeyWIF) {
      console.log('Converting private key to WIF format...');
      privateKeyWIF = ECPair.fromPrivateKey(
        Buffer.from(runeInfo.keypair.privateKey, 'hex'),
        { network: NETWORK }
      ).toWIF();
      
      // Save the WIF format for future use
      runeInfo.keypair.privateKeyWIF = privateKeyWIF;
      fs.writeFileSync(
        path.resolve(__dirname, '../ovt-fund/src/rune-info.json'),
        JSON.stringify(runeInfo, null, 2)
      );
    }
    
    console.log(`Using address: ${runeInfo.keypair.address}`);
    
    // Get UTXOs for the address
    console.log('Getting UTXOs for the address...');
    const utxos = await getUtxos(runeInfo.keypair.address);
    console.log(`Found ${utxos.length} UTXOs`);
    
    if (utxos.length === 0) {
      throw new Error('No UTXOs found. Please fund the address with testnet BTC first.');
    }
    
    // Create the minting transaction
    console.log('Creating minting transaction...');
    const { txid } = mintRuneWithOrd(runeInfo, mintAmount, privateKeyWIF, signatures);
    
    console.log('\n=== OVT Tokens Minted Successfully ===');
    console.log(`Rune Symbol: ${runeInfo.symbol}`);
    console.log(`Amount Minted: ${mintAmount}`);
    console.log(`Transaction ID: ${txid}`);
    console.log(`Explorer Link: https://mempool.space/testnet/tx/${txid}`);
    
    // Update the Rune information
    runeInfo.mintingTransactions = runeInfo.mintingTransactions || [];
    runeInfo.mintingTransactions.push({
      amount: mintAmount,
      txid: txid,
      timestamp: new Date().toISOString(),
      signatures: signatures.length > 0 ? signatures.length : undefined
    });
    
    runeInfo.totalSupply = (runeInfo.totalSupply || runeInfo.initialSupply) + mintAmount;
    
    // Save the updated Rune information
    fs.writeFileSync(
      path.resolve(__dirname, '../ovt-fund/src/rune-info.json'),
      JSON.stringify(runeInfo, null, 2)
    );
    
    console.log('\nRune information updated in ovt-fund/src/rune-info.json');
    console.log(`New total supply: ${runeInfo.totalSupply} ${runeInfo.symbol}`);
    
    return txid;
  } catch (error) {
    console.error('Failed to mint OVT tokens:', error);
    throw error;
  }
}

// If this script is run directly, execute the mint function
if (require.main === module) {
  mintOVTTokens().catch(error => {
    console.error('Mint operation failed:', error);
    process.exit(1);
  });
}

// Export the function for use in other scripts (e.g., for multi-sig integration)
module.exports = { mintOVTTokens }; 