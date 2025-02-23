import { useMemo, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import TokenExplorerModal from './TokenExplorerModal';
import { useOVTClient } from '../src/hooks/useOVTClient';
import { Portfolio } from '../src/hooks/useOVTClient';

// Constants for numeric handling
const SATS_PER_BTC = 100000000;

interface NAVData {
  name: string;
  value: number;      // Initial value in sats
  current: number;    // Current value in sats
  change: number;     // Percentage change
  description: string;
}

interface NAVVisualizationProps {
  data: Portfolio[];
  totalValue: string;
  changePercentage: string;
  baseCurrency?: 'usd' | 'btc';
}

const formatCurrencyValue = (value: number, currency: 'usd' | 'btc' = 'usd', btcPrice: number | null = null) => {
  if (currency === 'usd') {
    // Convert from sats to USD if btcPrice is provided
    const usdValue = btcPrice ? (value / SATS_PER_BTC) * btcPrice : value;
    
    // Use 'M' for values ≥ 1M with one decimal
    if (usdValue >= 1000000) {
      return `$${(usdValue / 1000000).toFixed(1)}M`;
    }
    // Use 'k' for values ≥ 1k with no decimals
    if (usdValue >= 1000) {
      return `$${Math.floor(usdValue / 1000)}k`;
    }
    // Show full number with no decimals for values ≥ 100
    if (usdValue >= 100) {
      return `$${Math.floor(usdValue)}`;
    }
    // Show full number with two decimals for values < 100
    return `$${usdValue.toFixed(2)}`;
  } else {
    const sats = Math.floor(value);
    // For values >= 10M sats (0.1 BTC), display in BTC with 2 decimals
    if (sats >= 10000000) {
      return `₿${(sats / SATS_PER_BTC).toFixed(2)}`;
    }
    // For values >= 1M sats, use 'M' notation with two decimals
    if (sats >= 1000000) {
      return `${(sats / 1000000).toFixed(2)}M sats`;
    }
    // For values >= 1k sats, use 'k' notation with no decimals
    if (sats >= 1000) {
      return `${Math.floor(sats / 1000)}k sats`;
    }
    // Show full number for values < 1k sats
    return `${sats} sats`;
  }
};

interface CustomTooltipProps {
  active?: boolean;
  payload?: any[];
  label?: string;
  currency: 'usd' | 'btc';
  btcPrice: number | null;
}

const CustomTooltip = ({ active, payload, label, currency, btcPrice }: CustomTooltipProps) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    const initialValue = data.initial;
    const growthValue = data.growth;
    const totalValue = initialValue + growthValue;
    const growthPercentage = ((growthValue / initialValue) * 100).toFixed(1);

    return (
      <div className="bg-white p-4 rounded-lg shadow-lg border border-gray-200">
        <h3 className="font-semibold text-gray-900">{data.name}</h3>
        <p className="text-sm text-gray-600 mt-1">{data.description}</p>
        <div className="mt-2 space-y-1">
          <p className="text-lg font-medium">
            Total: {formatCurrencyValue(totalValue, currency, btcPrice)}
          </p>
          <div className="flex items-center space-x-2">
            <span className="text-gray-600">
              Initial: {formatCurrencyValue(initialValue, currency, btcPrice)}
            </span>
            <span className={`text-sm ${growthValue >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              ({growthValue >= 0 ? '+' : ''}{growthPercentage}%)
            </span>
          </div>
          <p className="text-sm text-gray-600">
            Growth: {formatCurrencyValue(growthValue, currency, btcPrice)}
          </p>
        </div>
      </div>
    );
  }
  return null;
};

const formatDate = (date: Date): string => {
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
};

const getInitialTransaction = (tokenAmount: number, pricePerToken: number, currency: 'usd' | 'btc' = 'usd') => {
  const initialDate = new Date('2024-01-15'); // Set a fixed initial investment date
  return {
    date: formatDate(initialDate),
    type: 'buy' as const,
    amount: new Intl.NumberFormat('en-US').format(tokenAmount),
    price: formatCurrencyValue(pricePerToken, currency), // Use actual price per token
  };
};

export default function NAVVisualization({ data, totalValue, changePercentage, baseCurrency = 'usd' }: NAVVisualizationProps) {
  const [selectedToken, setSelectedToken] = useState<Portfolio | null>(null);
  const { formatValue, btcPrice } = useOVTClient();

  const formattedData = useMemo(() => {
    console.log('Formatting data:', data);
    return data.map(item => {
      const initial = Number(item.value);
      const current = Number(item.current);
      const growth = current - initial;
      
      // Keep values in sats and let formatCurrencyValue handle the conversion
      return {
        ...item,
        name: item.name,
        initial,
        growth,
        total: initial + growth,
        change: Number(((growth / initial) * 100).toFixed(1)),
      };
    });
  }, [data]);

  // Use the same NAV value from the card
  const chartNAV = useMemo(() => {
    console.log('Calculating chart NAV, total value:', totalValue, 'btcPrice:', btcPrice, 'baseCurrency:', baseCurrency);
    return totalValue;
  }, [totalValue]);

  const formatYAxis = (value: number) => {
    console.log('Formatting Y axis value:', value, 'in mode:', baseCurrency);
    return formatCurrencyValue(value, baseCurrency, btcPrice);
  };

  // Calculate the maximum value for proper Y-axis scaling
  const maxValue = formattedData.length > 0 
    ? Math.max(...formattedData.map(item => item.total))
    : 1000000; // Default to 1M when no data

  const yAxisDomain = [0, Math.ceil(maxValue * 1.1)]; // Add 10% padding to the top
  console.log('Y axis domain:', yAxisDomain);

  // If no data, provide a default dataset
  const chartData = formattedData.length > 0 ? formattedData : [
    {
      name: 'No Data',
      initial: 0,
      growth: 0,
      total: 0,
      change: 0,
      description: 'No portfolio data available'
    }
  ];

  const handleClick = (data: any, index: number) => {
    console.log('Bar clicked:', data);
    const token = formattedData.find(item => item.name === data.name);
    if (token) {
      setSelectedToken(token);
    }
  };

  return (
    <div className="card">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-semibold">OTORI Net Asset Value - Tracked by $OVT</h2>
        <div className="text-right">
          <p className="text-2xl font-bold">{chartNAV}</p>
          <p className="text-sm text-green-500">{changePercentage}</p>
        </div>
      </div>
      
      <div className="h-80">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart
            data={chartData}
            margin={{
              top: 20,
              right: 30,
              left: 30,
              bottom: 10,
            }}
          >
            <CartesianGrid 
              strokeDasharray="3 3" 
              horizontal={true}
              vertical={false}
              horizontalPoints={Array.from({ length: 5 }, (_, i) => (maxValue * 1.1 * (i + 1)) / 6)}
            />
            <XAxis 
              dataKey="name"
              padding={{ left: 20, right: 20 }}
            />
            <YAxis 
              tickFormatter={formatYAxis}
              domain={yAxisDomain}
              padding={{ top: 20 }}
            />
            <Tooltip content={(props) => <CustomTooltip {...props} currency={baseCurrency} btcPrice={btcPrice} />} />
            <Bar 
              dataKey="initial" 
              stackId="a" 
              fill="#93C5FD" 
              name="Initial" 
              onClick={handleClick}
              style={{ cursor: 'pointer' }}
            />
            <Bar 
              dataKey="growth" 
              stackId="a" 
              fill="#3B82F6" 
              name="Growth" 
              onClick={handleClick}
              style={{ cursor: 'pointer' }}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
      
      <div className="mt-4 flex items-center justify-end space-x-4">
        <div className="flex items-center">
          <div className="w-3 h-3 bg-blue-300 rounded-sm mr-2" />
          <span className="text-sm text-gray-600">Initial</span>
        </div>
        <div className="flex items-center">
          <div className="w-3 h-3 bg-blue-600 rounded-sm mr-2" />
          <span className="text-sm text-gray-600">Growth</span>
        </div>
      </div>

      <TokenExplorerModal
        isOpen={selectedToken !== null}
        onClose={() => setSelectedToken(null)}
        tokenData={selectedToken ? {
          name: selectedToken.name,
          description: selectedToken.description,
          initial: selectedToken.value,
          current: selectedToken.current,
          change: selectedToken.change,
          address: selectedToken.address || 'bc1p...',
          holdings: formatValue(selectedToken.current, baseCurrency),
          transactions: [getInitialTransaction(selectedToken.tokenAmount, selectedToken.pricePerToken, baseCurrency)]
        } : {
          name: '',
          description: '',
          initial: 0,
          current: 0,
          change: 0,
          address: '',
          holdings: '',
          transactions: []
        }}
        baseCurrency={baseCurrency}
      />
    </div>
  );
} 