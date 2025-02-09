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
    value: 200000, // Initial $200k
    current: 1040000, // 420% growth = $1.04M total
    change: 420,
    description: 'Encryption Layer'
  },
  {
    name: 'VoltFi',
    value: 150000, // Initial $150k
    current: 525000, // 250% growth = $525k total
    change: 250,
    description: 'Bitcoin Volatility Index on Bitcoin'
  },
  {
    name: 'MIXDTape',
    value: 100000, // Initial $100k
    current: 250000, // 150% growth = $250k total
    change: 150,
    description: 'Phygital Music for superfans - disrupting Streaming'
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
    totalValue: '$1.815M',
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
          totalValue: `$${(totalValue / 1000000).toFixed(3)}M`,
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

    fetchNAV(); // Initial fetch
    const interval = setInterval(fetchNAV, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, []);

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