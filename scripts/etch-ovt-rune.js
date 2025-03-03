/**
 * Script to etch an OVT Rune on Bitcoin testnet
 * 
 * This script creates a Rune with the symbol "OVT" and configures it
 * for open-ended minting to support rolling raises.
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

// Configuration for the OVT Rune
const RUNE_CONFIG = {
  symbol: 'OVT',
  decimals: 8,
  supply: 500000, // Initial supply of 500,000 OVT
  limit: 0,       // 0 means no limit (open-ended minting)
  terms: 'OTORI Vision Token - Testnet',
  spacers: 0,
  divisibility: 8,
  mint: true      // Allow future minting
};

// Testnet API endpoints
const TESTNET_API = {
  faucet: 'https://testnet-faucet.mempool.co/api/faucet',
  broadcast: 'https://mempool.space/testnet/api/tx',
  utxos: (address) => `https://mempool.space/testnet/api/address/${address}/utxo`
};

/**
 * Generate a new Bitcoin testnet keypair
 */
function generateKeypair() {
  const keypair = ECPair.makeRandom({ network: NETWORK });
  const { address } = bitcoinjs.payments.p2wpkh({
    pubkey: keypair.publicKey,
    network: NETWORK
  });
  
  return {
    privateKey: keypair.privateKey.toString('hex'),
    publicKey: keypair.publicKey.toString('hex'),
    address
  };
}

/**
 * Request testnet coins from a faucet
 */
async function requestFromFaucet(address) {
  try {
    const response = await axios.post(TESTNET_API.faucet, { address });
    console.log(`Faucet request successful: ${response.data.txid}`);
    return response.data.txid;
  } catch (error) {
    console.error('Failed to request from faucet:', error.response?.data || error.message);
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
 * Create a Rune etching transaction using ord
 */
function etchRuneWithOrd(runeConfig, privateKeyWIF) {
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
    
    // Create a temporary wallet for etching
    const walletName = `ovt-etch-${Date.now()}`;
    console.log(`Creating temporary wallet: ${walletName}`);
    
    // Import the private key to the wallet
    execSync(`ord --testnet wallet create ${walletName}`);
    execSync(`ord --testnet wallet import ${walletName} ${privateKeyWIF}`);
    
    // Construct the etch command
    const etchCommand = `ord --testnet wallet etch ${walletName} --fee-rate 10 --rune ${runeConfig.symbol} --supply ${runeConfig.supply} --divisibility ${runeConfig.divisibility} --spacers ${runeConfig.spacers} --terms "${runeConfig.terms}" ${runeConfig.mint ? '--mint' : ''}`;
    
    console.log(`Executing etch command: ${etchCommand}`);
    const result = execSync(etchCommand).toString();
    console.log('Etch result:', result);
    
    // Extract the transaction ID from the result
    const txidMatch = result.match(/[a-f0-9]{64}/);
    if (!txidMatch) {
      throw new Error('Could not extract transaction ID from etch result');
    }
    
    const txid = txidMatch[0];
    console.log(`Extracted transaction ID: ${txid}`);
    
    return {
      txid,
      command: etchCommand,
      result
    };
  } catch (error) {
    console.error('Failed to etch Rune with ord:', error.message);
    throw error;
  }
}

/**
 * Main function to etch the OVT Rune
 */
async function etchOVTRune() {
  try {
    console.log('Generating keypair for OVT Rune...');
    const keypair = generateKeypair();
    console.log(`Generated address: ${keypair.address}`);
    
    console.log('Requesting testnet coins from faucet...');
    await requestFromFaucet(keypair.address);
    
    // Wait for the faucet transaction to confirm
    console.log('Waiting for faucet transaction to confirm (30 seconds)...');
    await new Promise(resolve => setTimeout(resolve, 30000));
    
    console.log('Getting UTXOs for the address...');
    const utxos = await getUtxos(keypair.address);
    console.log(`Found ${utxos.length} UTXOs`);
    
    if (utxos.length === 0) {
      throw new Error('No UTXOs found. Please try again later.');
    }
    
    // Convert private key to WIF format for ord
    const privateKeyWIF = ECPair.fromPrivateKey(
      Buffer.from(keypair.privateKey, 'hex'),
      { network: NETWORK }
    ).toWIF();
    
    console.log('Creating Rune etching transaction with ord...');
    const { txid } = etchRuneWithOrd(RUNE_CONFIG, privateKeyWIF);
    
    console.log('\n=== OVT Rune Etched Successfully ===');
    console.log(`Rune Symbol: ${RUNE_CONFIG.symbol}`);
    console.log(`Initial Supply: ${RUNE_CONFIG.supply}`);
    console.log(`Minting Enabled: ${RUNE_CONFIG.mint ? 'Yes' : 'No'}`);
    console.log(`Transaction ID: ${txid}`);
    console.log(`Explorer Link: https://mempool.space/testnet/tx/${txid}`);
    
    // Save the Rune information to a file
    const runeInfo = {
      symbol: RUNE_CONFIG.symbol,
      initialSupply: RUNE_CONFIG.supply,
      mintingEnabled: RUNE_CONFIG.mint,
      txid: txid,
      keypair: {
        address: keypair.address,
        privateKey: keypair.privateKey,
        privateKeyWIF: privateKeyWIF
      },
      createdAt: new Date().toISOString()
    };
    
    fs.writeFileSync(
      path.resolve(__dirname, '../ovt-fund/src/rune-info.json'),
      JSON.stringify(runeInfo, null, 2)
    );
    
    console.log('\nRune information saved to ovt-fund/src/rune-info.json');
    console.log('IMPORTANT: Keep this file secure as it contains the private key for minting.');
    
  } catch (error) {
    console.error('Failed to etch OVT Rune:', error);
  }
}

// Run the script
etchOVTRune(); 