import { useState, useCallback, useEffect } from 'react';
import { ArchClient } from '../lib/archClient';
import { useBitcoinPrice } from '../hooks/useBitcoinPrice';

// Constants for numeric handling
const SATS_PER_BTC = 100000000;

export interface Portfolio {
  name: string;
  value: number;      // in sats
  current: number;    // in sats
  change: number;     // percentage
  description: string;
  transactionId?: string;  // Reference to the position entry transaction
  tokenAmount: number;     // Number of tokens
  pricePerToken: number;   // Price per token in sats
}

interface NAVData {
  totalValue: string;         // Formatted string for display
  totalValueSats: number;     // Raw value in sats
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

// Update ArchClient interface to include required methods
interface ArchClientResponse {
  portfolioItems: Portfolio[];
}

// Extend ArchClient type
interface ArchClientType {
  getCurrentNAV(): Promise<ArchClientResponse>;
  addPosition(position: Portfolio): Promise<Portfolio>;
}

// Initialize the Arch client
const archClient: ArchClientType = new ArchClient({
  programId: process.env.NEXT_PUBLIC_PROGRAM_ID || '',
  treasuryAddress: process.env.NEXT_PUBLIC_TREASURY_ADDRESS || '',
  endpoint: process.env.NEXT_PUBLIC_ARCH_ENDPOINT || 'http://localhost:8000',
});

// Helper function to format values consistently
const formatValue = (sats: number, displayMode: 'btc' | 'usd' = 'btc', btcPrice?: number | null): string => {
  if (displayMode === 'usd' && btcPrice) {
    const usdValue = (sats / SATS_PER_BTC) * btcPrice;
    // USD formatting
    if (usdValue >= 1000000) {
      return `$${(usdValue / 1000000).toFixed(2)}M`;
    }
    if (usdValue >= 1000) {
      return `$${(usdValue / 1000).toFixed(2)}k`;
    }
    return `$${usdValue.toFixed(2)}`;
  }

  // BTC display mode
  const btcValue = sats / SATS_PER_BTC;
  if (sats >= 10000000) { // 0.1 BTC or more
    return `₿${btcValue.toFixed(2)}`;
  }
  if (sats >= 1000000) {
    return `${(sats / 1000000).toFixed(1)}M sats`;
  }
  if (sats >= 1000) {
    return `${(sats / 1000).toFixed(0)}k sats`;
  }
  return `${sats} sats`;
};

// Import mock data
import mockPortfolioData from '../mock-data/portfolio-positions.json';
import mockTokenData from '../mock-data/token-data.json';

// Store positions in memory for development
let portfolioPositions: Portfolio[] = [];

// Store transactions in memory for development
let mockTransactions: Transaction[] = [];

// Load initial positions from JSON file in mock mode
if (process.env.NEXT_PUBLIC_MOCK_MODE === 'true') {
  try {
    console.log('Mock mode enabled, loading portfolio positions...');
    portfolioPositions = mockPortfolioData;
    
    // Create transaction entries for each position
    mockTransactions = portfolioPositions.map(position => ({
      txid: position.transactionId || `position_${Date.now()}`,
      type: 'position_entry',
      amount: position.value,
      timestamp: Date.now(),
      status: 'confirmed',
      details: {
        position: position.name,
        currency: 'BTC'
      }
    }));

    // Add OVT mint transaction if it exists
    if (mockTokenData && mockTokenData.transactions) {
      mockTransactions.push(...mockTokenData.transactions);
    }

    console.log('Loaded portfolio positions:', portfolioPositions);
    console.log('Created mock transactions:', mockTransactions);
  } catch (err) {
    console.warn('Failed to load mock portfolio positions:', err);
    portfolioPositions = [];
    mockTransactions = [];
  }
} else {
  console.log('Mock mode not enabled. NEXT_PUBLIC_MOCK_MODE =', process.env.NEXT_PUBLIC_MOCK_MODE);
}

export function useOVTClient() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { price: btcPrice } = useBitcoinPrice();
  const [navData, setNavData] = useState<NAVData>(() => ({
    totalValue: '₿0.00',
    totalValueSats: 0,
    changePercentage: '+0%',
    portfolioItems: []
  }));

  // Get transaction history
  const getTransactionHistory = useCallback(async () => {
    if (process.env.NEXT_PUBLIC_MOCK_MODE === 'true') {
      return mockTransactions;
    }
    // In production, this would fetch from Arch Network
    return [];
  }, []);

  // Fetch NAV data periodically
  useEffect(() => {
    const fetchNAV = async () => {
      try {
        // In development, use the in-memory positions
        if (process.env.NEXT_PUBLIC_MOCK_MODE === 'true') {
          console.log('Fetching NAV in mock mode');
          console.log('Current portfolio positions:', portfolioPositions);
          
          // Calculate totals in sats (excluding OVT mint)
          const totalValueSats = portfolioPositions.reduce((sum, item) => {
            console.log(`Adding ${item.name} value: ${item.current} sats`);
            return sum + item.current;
          }, 0);
          
          const totalInitialSats = portfolioPositions.reduce((sum, item) => {
            console.log(`Adding ${item.name} initial: ${item.value} sats`);
            return sum + item.value;
          }, 0);

          console.log('Total value in sats:', totalValueSats, '=', totalValueSats / SATS_PER_BTC, 'BTC');
          console.log('Total initial in sats:', totalInitialSats, '=', totalInitialSats / SATS_PER_BTC, 'BTC');
          
          const changePercentage = totalInitialSats > 0 ? 
            ((totalValueSats - totalInitialSats) / totalInitialSats) * 100 : 0;

          // Update current values and changes for each position
          const updatedPositions = portfolioPositions.map(position => ({
            ...position,
            current: position.value, // For now, current equals initial
            change: 0 // For now, no change
          }));

          const formattedTotal = formatValue(totalValueSats, 'btc');
          console.log('Formatted total:', formattedTotal);
          
          const newNavData = {
            totalValue: formattedTotal,
            totalValueSats,
            changePercentage: `+${changePercentage.toFixed(0)}%`,
            portfolioItems: updatedPositions
          };
          
          console.log('Setting new NAV data:', newNavData);
          setNavData(newNavData);
          return;
        }

        // In production, fetch from Arch Network
        console.log('Fetching NAV from Arch Network');
        const nav = await archClient.getCurrentNAV();
        const portfolioItems = nav.portfolioItems.map(item => ({
          name: item.name,
          value: item.value,
          current: item.current,
          change: item.change,
          description: item.description,
          transactionId: item.transactionId,
          tokenAmount: item.tokenAmount,
          pricePerToken: item.pricePerToken
        }));

        const totalValueSats = portfolioItems.reduce((sum, item) => sum + item.current, 0);
        const totalInitialSats = portfolioItems.reduce((sum, item) => sum + item.value, 0);
        const changePercentage = ((totalValueSats - totalInitialSats) / totalInitialSats) * 100;
        
        setNavData({
          totalValue: formatValue(totalValueSats, 'btc'),
          totalValueSats,
          changePercentage: `+${changePercentage.toFixed(0)}%`,
          portfolioItems
        });
      } catch (err) {
        console.error('Failed to fetch NAV:', err);
      }
    };

    fetchNAV();
    const interval = setInterval(fetchNAV, 30000);

    return () => clearInterval(interval);
  }, []);

  // Add position entry
  const addPosition = useCallback(async (position: Omit<Portfolio, 'current' | 'change'>) => {
    const newPosition: Portfolio = {
      ...position,
      current: position.value, // Initially, current value equals initial value
      change: 0
    };

    if (process.env.NEXT_PUBLIC_MOCK_MODE === 'true') {
      portfolioPositions.push(newPosition);
      return newPosition;
    }

    // In production, this would be stored on-chain
    return await archClient.addPosition(newPosition);
  }, []);

  return {
    isLoading,
    error,
    navData,
    btcPrice,
    formatValue: (sats: number, displayMode: 'btc' | 'usd' = 'btc') => 
      formatValue(sats, displayMode, btcPrice),
    addPosition,
    getPositions: () => portfolioPositions,
    getTransactionHistory
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