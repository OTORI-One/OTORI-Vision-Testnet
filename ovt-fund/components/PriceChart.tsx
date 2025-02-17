import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { useBitcoinPrice } from '../src/hooks/useBitcoinPrice';

interface PriceData {
  name: string;
  value: number;
}

interface PriceChartProps {
  data?: PriceData[];
  baseCurrency?: 'usd' | 'btc';
}

const formatCurrencyValue = (value: number, currency: 'usd' | 'btc' = 'usd', btcPrice: number | null = null) => {
  if (currency === 'usd') {
    if (value >= 1000000) return `$${(value / 1000000).toFixed(2)}M`;
    if (value >= 1000) return `$${(value / 1000).toFixed(0)}k`;
    return `$${value.toFixed(2)}`;
  } else {
    const currentBtcPrice = btcPrice || 40000; // Use real BTC price if available
    const sats = Math.floor((value / currentBtcPrice) * 100000000);
    if (sats >= 1000000000) return `₿${(sats / 100000000).toFixed(2)}`;
    if (sats >= 1000000) return `${(sats / 1000000).toFixed(1)}M sats`;
    if (sats >= 1000) return `${(sats / 1000).toFixed(0)}k sats`;
    return `${sats} sats`;
  }
};

const CustomTooltip = ({ active, payload, label, currency, btcPrice }: any) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload;
    return (
      <div className="bg-white p-4 rounded-lg shadow-lg border border-gray-200">
        <p className="text-sm text-gray-600">{data.name}</p>
        <p className="text-lg font-semibold mt-1">
          {formatCurrencyValue(data.value, currency, btcPrice)}
        </p>
      </div>
    );
  }
  return null;
};

const mockData = [
  { name: 'Q1 \'26', value: 200 },
  { name: 'Q2 \'26', value: 400 },
  { name: 'Q3 \'26', value: 300 },
  { name: 'Q4 \'26', value: 280 },
];

export default function PriceChart({ data = mockData, baseCurrency = 'usd' }: PriceChartProps) {
  const { price: btcPrice } = useBitcoinPrice();

  const formatYAxis = (value: number) => {
    return formatCurrencyValue(value, baseCurrency, btcPrice);
  };

  // Calculate the maximum value for proper Y-axis scaling
  const maxValue = Math.max(...data.map(item => item.value));
  const yAxisDomain = [0, Math.ceil(maxValue * 1.1)]; // Add 10% padding to the top

  return (
    <div className="bg-white p-6 rounded-lg shadow">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold text-gray-700">Price Performance</h2>
      </div>
      <div className="h-80">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart 
            data={data}
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
            <Line
              type="monotone"
              dataKey="value"
              stroke="#4F46E5"
              strokeWidth={2}
              dot={{ r: 4 }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
} 