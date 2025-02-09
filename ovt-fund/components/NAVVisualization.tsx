import { useMemo, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import TokenExplorerModal from './TokenExplorerModal';

interface NAVData {
  name: string;
  initial: number;
  current: number;
  change: number;
  description: string;
}

interface NAVVisualizationProps {
  data: NAVData[];
  totalValue: string;
  changePercentage: string;
  baseCurrency?: 'usd' | 'btc';
}

const formatCurrencyValue = (value: number, currency: 'usd' | 'btc' = 'usd') => {
  if (currency === 'usd') {
    if (value >= 1000000) return `$${(value / 1000000).toFixed(2)}M`;
    if (value >= 1000) return `$${(value / 1000).toFixed(0)}k`;
    return `$${value.toFixed(2)}`;
  } else {
    const btcPrice = 40000; // TODO: Get real BTC price
    const sats = Math.floor((value / btcPrice) * 100000000);
    if (sats >= 1000000000) return `₿${(sats / 100000000).toFixed(2)}`;
    if (sats >= 1000000) return `${(sats / 1000000).toFixed(1)}M sats`;
    if (sats >= 1000) return `${(sats / 1000).toFixed(0)}k sats`;
    return `${sats} sats`;
  }
};

const CustomTooltip = ({ active, payload, label, currency }: any) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    const totalValue = data.current;
    const delta = totalValue - data.initial;
    const deltaPercentage = ((delta / data.initial) * 100).toFixed(0);

    return (
      <div className="bg-white p-4 rounded-lg shadow-lg border border-gray-200">
        <h3 className="font-semibold text-gray-900">{data.name}</h3>
        <p className="text-sm text-gray-600 mt-1">{data.description}</p>
        <div className="mt-2 space-y-1">
          <p className="text-lg font-medium">
            Total: {formatCurrencyValue(totalValue, currency)}
          </p>
          <div className="flex items-center space-x-2">
            <span className={delta >= 0 ? 'text-green-600' : 'text-red-600'}>
              {delta >= 0 ? '+' : ''}{formatCurrencyValue(Math.abs(delta), currency)}
            </span>
            <span className={`text-sm ${delta >= 0 ? 'text-green-600' : 'text-red-600'}`}>
              ({delta >= 0 ? '+' : ''}{deltaPercentage}%)
            </span>
          </div>
          <p className="text-sm text-gray-600">
            Initial: {formatCurrencyValue(data.initial, currency)}
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
  const [selectedToken, setSelectedToken] = useState<NAVData | null>(null);

  const formattedData = useMemo(() => {
    return data.map(item => ({
      ...item,
      initial: Number(item.initial.toFixed(2)),
      current: Number(item.current.toFixed(2)),
    }));
  }, [data]);

  const formatYAxis = (value: number) => {
    return formatCurrencyValue(value, baseCurrency);
  };

  // Calculate the maximum value for proper Y-axis scaling
  const maxValue = Math.max(...data.map(item => item.current));
  const yAxisDomain = [0, Math.ceil(maxValue * 1.1)]; // Add 10% padding to the top

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
          <p className="text-2xl font-bold">{totalValue}</p>
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
              name="Current" 
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
          <span className="text-sm text-gray-600">Current</span>
        </div>
      </div>

      {selectedToken && (
        <TokenExplorerModal
          isOpen={!!selectedToken}
          onClose={() => setSelectedToken(null)}
          tokenData={{
            ...selectedToken,
            address: '0x1234...5678', // Mock address
            holdings: `${new Intl.NumberFormat('en-US').format(selectedToken.initial)} tokens`,
            transactions: [getInitialTransaction(selectedToken.initial, baseCurrency)],
          }}
        />
      )}
    </div>
  );
} 