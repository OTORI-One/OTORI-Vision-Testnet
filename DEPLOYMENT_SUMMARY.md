# OTORI Vision Token (OVT) Deployment Summary

## Deployment Status

✅ **Successfully Deployed to Bitcoin Testnet**

## Deployment Details

- **Program ID**: `a69a7dd583609c1e9f78771753592639376676872f9500552d77c9b13821b19b`
- **Network**: Bitcoin Testnet
- **Deployment Date**: February 27, 2025

## Deployment Process

1. **Build Process**:
   - The program was built in the `arch-sdk-testnet` directory
   - Build artifacts were generated in `arch-sdk-testnet/build_testnet/`

2. **Deployment**:
   - The program was deployed using the local `arch-cli` tool
   - The deployment command was executed from the project root directory
   - Despite a network validation warning, the deployment proceeded successfully

3. **Program ID Extraction**:
   - The program ID was extracted from the keypair file using a Node.js script
   - The extracted program ID is in hexadecimal format

4. **Frontend Configuration**:
   - The frontend configuration was updated in `ovt-fund/.env.local`
   - The mock mode was disabled to use the actual deployed contract

5. **Portfolio Data Population**:
   - Initial portfolio positions were populated using the `populate-initial-positions.ts` script
   - Three investment positions were created: Polymorphic Labs, VoltFi, and MIXDTape
   - Mock data was saved to `ovt-fund/src/mock-data/portfolio-positions.json`

6. **Token Minting**:
   - Initial OVT tokens were minted using the `mint-initial-ovt.ts` script
   - 500,000 OVT tokens were created with a price of $1 per token
   - Total value: 5.26315789 BTC (526,315,789 sats) or $500,000 USD
   - Token data was saved to `ovt-fund/src/mock-data/token-data.json`

## Running the Frontend

To start the frontend application:

```bash
cd ovt-fund
npm run dev
```

The application will be available at `http://localhost:3000`.

## Portfolio Details

The following portfolio positions were created:

1. **Polymorphic Labs**
   - Initial Investment: ₿1.50 (150,000,000 sats)
   - Token Amount: 500,000 tokens
   - Price Per Token: 300 sats
   - Description: Encryption Layer

2. **VoltFi**
   - Initial Investment: ₿0.88 (87,500,000 sats)
   - Token Amount: 350,000 tokens
   - Price Per Token: 250 sats
   - Description: Bitcoin Volatility Index on Bitcoin

3. **MIXDTape**
   - Initial Investment: ₿1.00 (100,000,000 sats)
   - Token Amount: 500,000 tokens
   - Price Per Token: 200 sats
   - Description: Phygital Music for superfans - disrupting Streaming

## Token Details

- **Total Supply**: 500,000 OVT tokens
- **Price Per Token**: 1,052 sats (approximately $1 USD)
- **Total Value**: 5.26315789 BTC (526,315,789 sats) or $500,000 USD

## Troubleshooting

If you encounter any issues with the deployment:

1. Verify that the program ID is correctly set in the `.env.local` file
2. Ensure that the `arch-cli` tool is properly installed and accessible
3. Check the network settings to ensure they match the deployment environment

## Next Steps

1. Test the application thoroughly on the testnet
2. Monitor the contract's performance and behavior
3. Prepare for mainnet deployment when ready 