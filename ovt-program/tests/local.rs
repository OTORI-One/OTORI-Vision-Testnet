use ovt_program::{
    mock_sdk::{
        program::AccountMeta,
        Pubkey,
        test_utils::TestClient,
        AccountInfo,
    },
    OVTInstruction, OVTState,
};
use borsh::BorshSerialize;

#[test]
fn test_initialize() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TestClient::new();
    let program_id = Pubkey::new_unique();
    let authority_key = Pubkey::new_unique();
    
    // Create state account
    let state_account = client.create_account(program_id)?;

    // Create authority account with signer privileges
    client.accounts.insert(authority_key, AccountInfo::new(authority_key, true, false));

    // Initialize program
    let instruction = OVTInstruction::Initialize {
        treasury_pubkey_bytes: [0u8; 33],
    };

    client.process_transaction(
        program_id,
        vec![
            AccountMeta::new(state_account.key, true),
            AccountMeta::new_readonly(authority_key, true),
        ],
        borsh::to_vec(&instruction).unwrap(),
    )?;

    // Verify state was initialized correctly
    let state: OVTState = client.get_account_data(&state_account.key)?;
    assert_eq!(state.nav_sats, 0);
    assert_eq!(state.total_supply, 0);
    assert_eq!(state.last_nav_update, 0);

    Ok(())
}

#[test]
fn test_nav_update() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TestClient::new();
    let program_id = Pubkey::new_unique();
    let authority_key = Pubkey::new_unique();
    
    // Create state account
    let state_account = client.create_account(program_id)?;
    println!("Created state account: {:?}", state_account.key);

    // Create authority account with signer privileges
    client.accounts.insert(authority_key, AccountInfo::new(authority_key, true, false));
    println!("Created authority account: {:?}", authority_key);

    // Initialize first
    let instruction = OVTInstruction::Initialize {
        treasury_pubkey_bytes: [0u8; 33],
    };

    let accounts = vec![
        AccountMeta::new(state_account.key, true),
        AccountMeta::new_readonly(authority_key, true),
    ];

    client.process_transaction(
        program_id,
        accounts.clone(),
        borsh::to_vec(&instruction).unwrap(),
    )?;

    // Verify initial state
    let state: OVTState = client.get_account_data(&state_account.key)?;
    println!("After initialize: nav_sats={}", state.nav_sats);

    // Update NAV
    let new_nav = 1_000_000; // 1M sats
    let instruction = OVTInstruction::UpdateNAV {
        btc_price_sats: new_nav,
    };

    // Make sure we're using the same account setup for the update
    let accounts = vec![
        AccountMeta::new(state_account.key, true),
        AccountMeta::new_readonly(authority_key, true),
    ];

    println!("Before update: Using state account {:?}", state_account.key);
    client.process_transaction(
        program_id,
        accounts,
        borsh::to_vec(&instruction).unwrap(),
    )?;

    // Verify NAV was updated
    println!("Reading state from account {:?}", state_account.key);
    let state: OVTState = client.get_account_data(&state_account.key)?;
    println!("After update: nav_sats={}", state.nav_sats);
    assert_eq!(state.nav_sats, new_nav);

    Ok(())
} 