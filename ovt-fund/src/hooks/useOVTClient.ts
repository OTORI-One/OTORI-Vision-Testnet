import { useState, useCallback, useEffect } from 'react';
import { ArchClient } from '../lib/archClient';

interface Portfolio {
  name: string;
  value: number;
  current: number;
  change: number;
  description: string;
}

interface NAVData {
  totalValue: string;
  changePercentage: string;
  portfolioItems: Portfolio[];
}

interface Transaction {
  txid: string;
  type: 'mint' | 'burn' | 'transfer' | 'position_entry' | 'position_exit';
  amount: number;
  timestamp: number;
  status: 'pending' | 'confirmed' | 'failed';
  details: {
    reason?: string;
    position?: string;
    signatures?: string[];
    currency?: string;
  };
}

// Initialize the Arch client
const archClient = new ArchClient({
  programId: process.env.NEXT_PUBLIC_PROGRAM_ID || '',
  treasuryAddress: process.env.NEXT_PUBLIC_TREASURY_ADDRESS || '',
  endpoint: process.env.NEXT_PUBLIC_ARCH_ENDPOINT || 'http://localhost:8000',
});

// Mock initial portfolio data
const INITIAL_PORTFOLIO_ITEMS = [
  {
    name: 'Polymorphic Labs',
    value: 500000000, // Initial 500M sats
    current: 2600000000, // 2.6B sats (₿26.00)
    change: 420,
    description: 'Encryption Layer'
  },
  {
    name: 'VoltFi',
    value: 150000000, // Initial 150M sats
    current: 525000000, // 525M sats
    change: 250,
    description: 'Bitcoin Volatility Index on Bitcoin'
  },
  {
    name: 'MIXDTape',
    value: 100000000, // Initial 100M sats
    current: 250000000, // 250M sats
    change: 150,
    description: 'Phygital Music for superfans - disrupting Streaming'
  }
];

// Mock transaction data
const MOCK_TRANSACTIONS: Transaction[] = [
  {
    txid: 'mock_tx_1',
    type: 'mint',
    amount: 1000000,
    timestamp: Date.now() - 86400000, // 1 day ago
    status: 'confirmed',
    details: {
      reason: 'Initial token distribution',
      signatures: ['sig1', 'sig2', 'sig3']
    }
  },
  {
    txid: 'mock_tx_2',
    type: 'position_entry',
    amount: 5.25, // 5.25 BTC
    timestamp: Date.now() - 43200000, // 12 hours ago
    status: 'confirmed',
    details: {
      position: 'Polymorphic Labs',
      signatures: ['sig1', 'sig2', 'sig3'],
      currency: 'BTC'
    }
  },
  {
    txid: 'mock_tx_3',
    type: 'position_entry',
    amount: 150000, // $150,000 USD
    timestamp: Date.now() - 21600000, // 6 hours ago
    status: 'confirmed',
    details: {
      position: 'VoltFi',
      signatures: ['sig1', 'sig2', 'sig3'],
      currency: 'USD'
    }
  }
];

// Simulate portfolio value changes
function simulatePortfolioChange(items: Portfolio[]): Portfolio[] {
  return items.map(item => ({
    ...item,
    current: item.current * (1 + (Math.random() * 0.02 - 0.01)), // Random -1% to +1% change in current value
    change: Number(((item.current / item.value - 1) * 100).toFixed(1)), // Recalculate change percentage
  }));
}

export function useOVTClient() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [navData, setNavData] = useState<NAVData>({
    totalValue: '₿33.75', // Total of all current values (26 + 5.25 + 2.5)
    changePercentage: '+302%',
    portfolioItems: INITIAL_PORTFOLIO_ITEMS
  });

  // Fetch NAV data periodically
  useEffect(() => {
    const fetchNAV = async () => {
      try {
        const nav = await archClient.getCurrentNAV();
        const totalValue = nav.value;
        const averageChange = nav.portfolioItems.reduce((sum, item) => sum + item.change, 0) / nav.portfolioItems.length;
        
        setNavData({
          totalValue: `₿${(totalValue / 100000000).toFixed(2)}`, // Convert sats to BTC
          changePercentage: `${averageChange >= 0 ? '+' : ''}${averageChange.toFixed(1)}%`,
          portfolioItems: nav.portfolioItems.map(item => ({
            name: item.name,
            value: item.value,
            current: item.value * (1 + item.change / 100),
            change: item.change,
            description: getProjectDescription(item.name),
          }))
        });
      } catch (err) {
        console.error('Failed to fetch NAV:', err);
      }
    };

    fetchNAV();
    const interval = setInterval(fetchNAV, 30000);

    return () => clearInterval(interval);
  }, []);

  const getTransactionHistory = useCallback(async (): Promise<Transaction[]> => {
    if (process.env.NEXT_PUBLIC_MOCK_MODE === 'true') {
      return MOCK_TRANSACTIONS;
    }

    try {
      const history = await archClient.getTransactionHistory('all');
      return history.map(tx => ({
        txid: tx.txid,
        type: mapTransactionType(tx.type),
        amount: tx.amount,
        timestamp: tx.timestamp,
        status: mapTransactionStatus(tx.confirmations),
        details: tx.metadata || {}
      }));
    } catch (err) {
      console.error('Failed to fetch transaction history:', err);
      throw err;
    }
  }, []);

  // Helper function to map transaction types
  const mapTransactionType = (type: 'MINT' | 'BURN' | 'TRANSFER' | 'POSITION_ENTRY' | 'POSITION_EXIT'): Transaction['type'] => {
    switch (type) {
      case 'MINT': return 'mint';
      case 'BURN': return 'burn';
      case 'TRANSFER': return 'transfer';
      case 'POSITION_ENTRY': return 'position_entry';
      case 'POSITION_EXIT': return 'position_exit';
    }
  };

  // Helper function to map transaction status based on confirmations
  const mapTransactionStatus = (confirmations: number): 'pending' | 'confirmed' | 'failed' => {
    if (confirmations === 0) return 'pending';
    if (confirmations < 0) return 'failed';
    return 'confirmed';
  };

  const buyOVT = useCallback(async (amount: number) => {
    setIsLoading(true);
    setError(null);
    try {
      // In a real implementation, we would:
      // 1. Get the connected wallet address
      // 2. Wait for the Bitcoin payment transaction
      // 3. Use the payment txid to verify and complete the purchase
      const mockPaymentTxid = 'mock_txid_' + Date.now();
      const result = await archClient.buyOVT(amount, mockPaymentTxid, 'mock_wallet_address');
      
      if (!result.success) {
        throw new Error(result.error || 'Failed to buy OVT');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to buy OVT');
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const sellOVT = useCallback(async (amount: number) => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await archClient.sellOVT(amount, 'mock_wallet_address');
      
      if (!result.success) {
        throw new Error(result.error || 'Failed to sell OVT');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to sell OVT');
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    isLoading,
    error,
    navData,
    buyOVT,
    sellOVT,
    getTransactionHistory,
  };
}

// Helper function to get project descriptions
function getProjectDescription(name: string): string {
  const descriptions: Record<string, string> = {
    'Polymorphic Labs': 'Encryption Layer',
    'VoltFi': 'Bitcoin Volatility Index on Bitcoin',
    'MIXDTape': 'Phygital Music for superfans - disrupting Streaming',
  };
  return descriptions[name] || '';
} 