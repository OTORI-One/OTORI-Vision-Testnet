import { useMemo, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import TokenExplorerModal from './TokenExplorerModal';
import { useOVTClient } from '../src/hooks/useOVTClient';
import { Portfolio } from '../src/hooks/useOVTClient';

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
    if (value >= 1000000) return `$${(value / 1000000).toFixed(2)}M`;
    if (value >= 1000) return `$${(value / 1000).toFixed(2)}k`;
    return `$${value.toFixed(2)}`;
  } else {
    const sats = Math.floor(value);
    // For values >= 10M sats (0.1 BTC), display in BTC with 2 decimals
    if (sats >= 10000000) {
      return `₿${(sats / 100000000).toFixed(2)}`;
    }
    // For values >= 1M sats, use M format
    if (sats >= 1000000) {
      return `${(sats / 1000000).toFixed(1)}M sats`;
    }
    // For values >= 1000 sats, use k format
    if (sats >= 1000) {
      return `${(sats / 1000).toFixed(0)}k sats`;
    }
    return `${sats} sats`;
  }
};

const CustomTooltip = ({ active, payload, label, currency }: any) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    const initialValue = data.initial;
    const growthValue = data.current;
    const totalValue = data.total;
    const growthPercentage = ((growthValue / initialValue) * 100).toFixed(0);

    return (
      <div className="bg-white p-4 rounded-lg shadow-lg border border-gray-200">
        <h3 className="font-semibold text-gray-900">{data.name}</h3>
        <p className="text-sm text-gray-600 mt-1">{data.description}</p>
        <div className="mt-2 space-y-1">
          <p className="text-lg font-medium">
            Total: {formatCurrencyValue(totalValue, currency)}
          </p>
          <div className="flex items-center space-x-2">
            <span className="text-gray-600">
              Initial: {formatCurrencyValue(initialValue, currency)}
            </span>
            <span className={`text-sm ${growthValue >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              ({growthValue >= 0 ? '+' : ''}{growthPercentage}%)
            </span>
          </div>
          <p className="text-sm text-gray-600">
            Growth: {formatCurrencyValue(growthValue, currency)}
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

const getInitialTransaction = (value: number, currency: 'usd' | 'btc' = 'usd') => {
  const initialDate = new Date('2024-01-15'); // Set a fixed initial investment date
  return {
    date: formatDate(initialDate),
    type: 'buy' as const,
    amount: new Intl.NumberFormat('en-US').format(value),
    price: formatCurrencyValue(1, currency), // Initial price of $1
  };
};

export default function NAVVisualization({ data, totalValue, changePercentage, baseCurrency = 'usd' }: NAVVisualizationProps) {
  const [selectedToken, setSelectedToken] = useState<Portfolio | null>(null);
  const { formatValue } = useOVTClient();

  const formattedData = useMemo(() => {
    console.log('Formatting data:', data);
    return data.map(item => {
      const initial = Number(item.value);
      const current = Number(item.current);
      const growth = current - initial;
      
      return {
        ...item,
        name: item.name,
        initial,          // Initial investment in sats
        current: current, // Full current value
        total: current,   // Total current value
        change: Number(((growth / initial) * 100).toFixed(1)),
      };
    });
  }, [data]);

  // Use the same NAV value from the card
  const chartNAV = useMemo(() => {
    console.log('Calculating chart NAV, total value:', totalValue);
    return totalValue;
  }, [totalValue]);

  const formatYAxis = (value: number) => {
    console.log('Formatting Y axis value:', value, 'in mode:', baseCurrency);
    return formatValue(value, baseCurrency);
  };

  // Calculate the maximum value for proper Y-axis scaling
  const maxValue = Math.max(...formattedData.map(item => item.total));
  const yAxisDomain = [0, Math.ceil(maxValue * 1.1)]; // Add 10% padding to the top
  console.log('Y axis domain:', yAxisDomain);

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
            data={formattedData}
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
            <Tooltip content={(props) => <CustomTooltip {...props} currency={baseCurrency} />} />
            <Bar 
              dataKey="initial" 
              stackId="a" 
              fill="#93C5FD" 
              name="Initial" 
              onClick={handleClick}
              style={{ cursor: 'pointer' }}
            />
            <Bar 
              dataKey="current" 
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
          address: '0x1234...5678',
          holdings: `${new Intl.NumberFormat('en-US').format(selectedToken.tokenAmount)} tokens`,
          transactions: [getInitialTransaction(selectedToken.value, baseCurrency)],
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