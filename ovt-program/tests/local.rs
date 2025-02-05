use arch_sdk::{
    prelude::*,
    test_utils::*,
};
use ovt_program::{OVTProgram, TokenMetadata, OVTClient};

#[tokio::test]
async fn test_full_flow() {
    // Initialize test environment
    let test_client = TestClient::new();
    let program_id = Pubkey::new_unique();
    
    // Create program accounts
    let mint = test_client.create_mint().await;
    let metadata = test_client.create_account::<TokenMetadata>().await;
    let treasury = test_client.create_token_account(&mint).await;
    let authority = Keypair::new();

    // Create OVT client
    let client = OVTClient::new(
        &test_client,
        program_id,
        mint,
        metadata,
        treasury,
        authority.clone(),
    );

    // Initialize OVT token
    let initial_supply = 1_000_000;
    client.initialize(initial_supply).await.unwrap();

    // Create test accounts
    let buyer = Keypair::new();
    let buyer_token_account = test_client.create_token_account(&mint).await;
    
    // Test buying OVT
    println!("Testing OVT purchase...");
    let buy_amount = 1000;
    let payment_proof = vec![1, 2, 3, 4]; // Simulated payment
    client.buy_ovt(
        buyer_token_account,
        buy_amount,
        payment_proof,
    ).await.unwrap();

    // Verify purchase
    let buyer_balance = test_client.get_token_balance(&buyer_token_account).await.unwrap();
    assert_eq!(buyer_balance, buy_amount);
    println!("Purchase successful! Buyer balance: {}", buyer_balance);

    // Test selling OVT
    println!("Testing OVT sale...");
    let btc_address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".to_string();
    client.sell_ovt(
        buyer_token_account,
        buy_amount,
        btc_address,
    ).await.unwrap();

    // Verify sale
    let buyer_balance_after = test_client.get_token_balance(&buyer_token_account).await.unwrap();
    assert_eq!(buyer_balance_after, 0);
    println!("Sale successful! Buyer balance after: {}", buyer_balance_after);

    // Test NAV calculation
    println!("Testing NAV calculation...");
    client.update_nav().await.unwrap();
    
    // Get updated metadata
    let metadata_data = test_client.get_account_data::<TokenMetadata>(&metadata).await.unwrap();
    println!("Current NAV: {} sats", metadata_data.nav_sats);
    println!("Total supply: {}", metadata_data.total_supply);

    println!("All tests passed successfully!");
} 