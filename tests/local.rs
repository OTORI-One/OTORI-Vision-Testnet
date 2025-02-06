#[test]
fn test_nav_update() {
    let mut client = TestClient::new();
    let state_account = client.create_account();
    let treasury_account = client.create_account();

    // Initialize the program
    client.process_instruction(
        &[
            AccountMeta::new(state_account.key, true),
            AccountMeta::new(treasury_account.key, false),
        ],
        &OVTInstruction::Initialize {
            treasury_pubkey_bytes: treasury_account.key.to_bytes(),
        },
    )
    .unwrap();

    // Update NAV
    client.process_instruction(
        &[
            AccountMeta::new(state_account.key, true),
        ],
        &OVTInstruction::UpdateNAV {
            nav_sats: 1000,
        },
    )
    .unwrap();

    // Verify state
    let state: OVTState = client.get_account_data(&state_account.key).unwrap();
    assert_eq!(state.nav_sats, 1000);
} 