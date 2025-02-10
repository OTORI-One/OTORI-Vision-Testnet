use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::PublicKey;
use ovt_program::runes_client::RunesClient;

#[tokio::test]
async fn test_multisig_flow() {
    // Initialize client
    let client = RunesClient::new(
        bitcoin::Network::Regtest,
        "http://localhost:8332".to_string(),
        None,
    );

    // Generate 5 admin keypairs
    let secp = Secp256k1::new();
    let mut admin_pubkeys = Vec::new();
    let mut admin_privkeys = Vec::new();

    for _ in 0..5 {
        let (privkey, pubkey) = secp.generate_keypair(&mut rand::thread_rng());
        admin_pubkeys.push(PublicKey::new(pubkey));
        admin_privkeys.push(privkey);
    }

    // Test minting tokens
    let amount = 1_000_000;
    let signatures = vec![
        "sig1".to_string(),
        "sig2".to_string(),
        "sig3".to_string(),
    ];

    let mint_result = client.mint_tokens(
        amount,
        signatures.clone(),
        admin_pubkeys.clone(),
    ).await;
    assert!(mint_result.is_ok());

    // Test adding a post-TGE position
    let position = PortfolioPosition {
        name: "Test Project".to_string(),
        amount: 1000000,
        price_per_token: 100,
        currency_spent: 100000000,
        transaction_id: Some("txid123".to_string()),
        safe_inscription_id: None,
        entry_timestamp: 1677649200,
        position_type: PositionType::PostTGE,
        status: PositionStatus::Active,
    };

    let add_position_result = client.add_post_tge_position(
        position,
        &signatures,
        &admin_pubkeys,
    ).await;
    assert!(add_position_result.is_ok());

    // Test with insufficient signatures
    let insufficient_sigs = vec!["sig1".to_string(), "sig2".to_string()];
    let invalid_result = client.mint_tokens(
        amount,
        insufficient_sigs,
        admin_pubkeys.clone(),
    ).await;
    assert!(invalid_result.is_err());

    // Test with invalid admin keys
    let mut invalid_pubkeys = admin_pubkeys.clone();
    invalid_pubkeys.pop(); // Remove one key to make it invalid
    let invalid_keys_result = client.mint_tokens(
        amount,
        signatures,
        invalid_pubkeys,
    ).await;
    assert!(invalid_keys_result.is_err());
}

#[tokio::test]
async fn test_real_signatures() {
    let client = RunesClient::new(
        bitcoin::Network::Regtest,
        "http://localhost:8332".to_string(),
        None,
    );

    // Generate admin keys and messages
    let secp = Secp256k1::new();
    let message = b"Test mint 1000000 OVT";
    let message_hash = bitcoin::hashes::sha256::Hash::hash(message);

    // Generate and sign with 3 keys
    let mut signatures = Vec::new();
    let mut admin_pubkeys = Vec::new();

    for _ in 0..5 {
        let (privkey, pubkey) = secp.generate_keypair(&mut rand::thread_rng());
        admin_pubkeys.push(PublicKey::new(pubkey));
        
        if signatures.len() < 3 {
            let sig = secp.sign_ecdsa(&message_hash, &privkey);
            signatures.push(sig.to_string());
        }
    }

    // Verify multisig
    let result = client.verify_admin_multisig(
        &signatures,
        message,
        &admin_pubkeys,
    ).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
} 