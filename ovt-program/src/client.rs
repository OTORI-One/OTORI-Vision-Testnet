use arch_sdk::{
    prelude::*,
    client::{Client, ClientResult},
    transaction::Transaction,
};
use borsh::BorshSerialize;

use crate::{OVTInstruction, SAFEData};

pub struct OVTClient<'a> {
    client: &'a Client,
    program_id: Pubkey,
    mint: Pubkey,
    metadata: Pubkey,
    treasury: Pubkey,
    authority: Keypair,
}

impl<'a> OVTClient<'a> {
    pub fn new(
        client: &'a Client,
        program_id: Pubkey,
        mint: Pubkey,
        metadata: Pubkey,
        treasury: Pubkey,
        authority: Keypair,
    ) -> Self {
        Self {
            client,
            program_id,
            mint,
            metadata,
            treasury,
            authority,
        }
    }

    pub async fn initialize(&self, initial_supply: u64) -> ClientResult<Signature> {
        let instruction = OVTInstruction::Initialize { initial_supply };
        let accounts = vec![
            AccountMeta::new(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new(self.treasury, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn mint(&self, amount: u64) -> ClientResult<Signature> {
        let instruction = OVTInstruction::Mint { amount };
        let accounts = vec![
            AccountMeta::new(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new(self.treasury, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn burn(&self, amount: u64) -> ClientResult<Signature> {
        let instruction = OVTInstruction::Burn { amount };
        let accounts = vec![
            AccountMeta::new(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new(self.treasury, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn add_safe(&self, safe_data: SAFEData) -> ClientResult<Signature> {
        let instruction = OVTInstruction::AddSAFE { safe_data };
        let safes_pda = Pubkey::find_program_address(
            &[b"safes", self.mint.as_ref()],
            &self.program_id,
        ).0;

        let accounts = vec![
            AccountMeta::new(safes_pda, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn update_safe(&self, safe_id: u64, new_data: SAFEData) -> ClientResult<Signature> {
        let instruction = OVTInstruction::UpdateSAFE { safe_id, new_data };
        let safes_pda = Pubkey::find_program_address(
            &[b"safes", self.mint.as_ref()],
            &self.program_id,
        ).0;

        let accounts = vec![
            AccountMeta::new(safes_pda, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn update_nav(&self) -> ClientResult<Signature> {
        let instruction = OVTInstruction::UpdateNAV;
        let safes_pda = Pubkey::find_program_address(
            &[b"safes", self.mint.as_ref()],
            &self.program_id,
        ).0;
        let oracle_pda = Pubkey::find_program_address(
            &[b"oracle", self.mint.as_ref()],
            &self.program_id,
        ).0;

        let accounts = vec![
            AccountMeta::new(self.metadata, false),
            AccountMeta::new(self.treasury, false),
            AccountMeta::new(safes_pda, false),
            AccountMeta::new_readonly(oracle_pda, false),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn buy_ovt(
        &self,
        buyer_token_account: Pubkey,
        amount: u64,
        payment_proof: Vec<u8>,
    ) -> ClientResult<Signature> {
        let instruction = OVTInstruction::BuyOVT {
            ovt_amount: amount,
            payment_proof,
        };

        let oracle_pda = Pubkey::find_program_address(
            &[b"oracle", self.mint.as_ref()],
            &self.program_id,
        ).0;

        let accounts = vec![
            AccountMeta::new(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new(self.treasury, false),
            AccountMeta::new(buyer_token_account, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
            AccountMeta::new_readonly(oracle_pda, false),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }

    pub async fn sell_ovt(
        &self,
        seller_token_account: Pubkey,
        amount: u64,
        btc_address: String,
    ) -> ClientResult<Signature> {
        let instruction = OVTInstruction::SellOVT {
            ovt_amount: amount,
            btc_address,
        };

        let oracle_pda = Pubkey::find_program_address(
            &[b"oracle", self.mint.as_ref()],
            &self.program_id,
        ).0;

        let accounts = vec![
            AccountMeta::new(self.mint, false),
            AccountMeta::new(self.metadata, false),
            AccountMeta::new(self.treasury, false),
            AccountMeta::new(seller_token_account, false),
            AccountMeta::new_readonly(self.authority.pubkey(), true),
            AccountMeta::new_readonly(oracle_pda, false),
        ];

        let tx = Transaction::new_signed_with_payer(
            &[Instruction::new_with_borsh(
                self.program_id,
                &instruction,
                accounts,
            )],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            self.client.get_latest_blockhash().await?,
        );

        self.client.send_and_confirm_transaction(&tx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arch_sdk::test_utils::*;

    #[tokio::test]
    async fn test_buy_and_sell_ovt() {
        let test_client = TestClient::new();
        let program_id = Pubkey::new_unique();
        let mint = test_client.create_mint().await;
        let metadata = test_client.create_account::<TokenMetadata>().await;
        let treasury = test_client.create_token_account(&mint).await;
        let authority = Keypair::new();

        let client = OVTClient::new(
            &test_client,
            program_id,
            mint,
            metadata,
            treasury,
            authority,
        );

        // Initialize with some supply
        client.initialize(1_000_000).await.unwrap();

        // Create buyer account
        let buyer = Keypair::new();
        let buyer_token_account = test_client.create_token_account(&mint).await;

        // Buy OVT
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

        // Sell OVT
        let btc_address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".to_string();
        client.sell_ovt(
            buyer_token_account,
            buy_amount,
            btc_address,
        ).await.unwrap();

        // Verify sale
        let buyer_balance_after = test_client.get_token_balance(&buyer_token_account).await.unwrap();
        assert_eq!(buyer_balance_after, 0);
    }
} 