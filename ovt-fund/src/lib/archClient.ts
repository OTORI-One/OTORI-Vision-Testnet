import { PublicKey } from '@omnisat/lasereyes';

// Add a type declaration to fix the missing type
declare module '@omnisat/lasereyes' {
  export class PublicKey {
    toString(): string;
    toBuffer(): Buffer;
    equals(other: PublicKey): boolean;
  }
}

export interface ArchTransaction {
  txid: string;
  type: 'MINT' | 'BURN' | 'TRANSFER' | 'POSITION_ENTRY' | 'POSITION_EXIT';
  amount: number;
  confirmations: number;
  timestamp: number;
  metadata?: {
    reason?: string;
    position?: string;
    signatures?: string[];
    currency?: string;
  };
}

export interface NAVUpdate {
  timestamp: number;
  value: number;
  portfolioItems: {
    name: string;
    value: number;
    change: number;
  }[];
}

export class ArchClient {
  private programId: string;
  private treasuryAddress: string;
  private endpoint: string;

  constructor(config: {
    programId: string;
    treasuryAddress: string;
    endpoint: string;
  }) {
    this.programId = config.programId;
    this.treasuryAddress = config.treasuryAddress;
    this.endpoint = config.endpoint;
  }

  async verifyBitcoinPayment(txid: string, expectedAmount: number): Promise<boolean> {
    try {
      const response = await fetch(`${this.endpoint}/verify_payment`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          txid,
          expected_amount: expectedAmount,
          treasury_address: this.treasuryAddress,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to verify payment');
      }

      const result = await response.json();
      return result.verified;
    } catch (error) {
      console.error('Payment verification failed:', error);
      return false;
    }
  }

  async buyOVT(
    amount: number,
    paymentTxid: string,
    walletAddress: string
  ): Promise<{ success: boolean; txid?: string; error?: string }> {
    try {
      // Verify the payment first
      const isVerified = await this.verifyBitcoinPayment(paymentTxid, amount);
      if (!isVerified) {
        throw new Error('Payment verification failed');
      }

      // Execute the buyback instruction
      const response = await fetch(`${this.endpoint}/buy_ovt`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          program_id: this.programId,
          payment_txid: paymentTxid,
          amount,
          wallet_address: walletAddress,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to execute OVT purchase');
      }

      const result = await response.json();
      return {
        success: true,
        txid: result.txid,
      };
    } catch (error) {
      console.error('OVT purchase failed:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred',
      };
    }
  }

  async sellOVT(
    amount: number,
    walletAddress: string
  ): Promise<{ success: boolean; txid?: string; error?: string }> {
    try {
      const response = await fetch(`${this.endpoint}/sell_ovt`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          program_id: this.programId,
          amount,
          wallet_address: walletAddress,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to execute OVT sale');
      }

      const result = await response.json();
      return {
        success: true,
        txid: result.txid,
      };
    } catch (error) {
      console.error('OVT sale failed:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred',
      };
    }
  }

  async getCurrentNAV(): Promise<NAVUpdate> {
    try {
      const response = await fetch(`${this.endpoint}/get_nav`);
      if (!response.ok) {
        throw new Error('Failed to fetch NAV');
      }
      return await response.json();
    } catch (error) {
      console.error('Failed to fetch NAV:', error);
      throw error;
    }
  }

  async getTransactionHistory(walletAddress: string): Promise<ArchTransaction[]> {
    try {
      const response = await fetch(
        `${this.endpoint}/transactions?address=${walletAddress}`
      );
      if (!response.ok) {
        throw new Error('Failed to fetch transaction history');
      }
      return await response.json();
    } catch (error) {
      console.error('Failed to fetch transaction history:', error);
      throw error;
    }
  }
} 