import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface PriceData {
  name: string;
  value: number;
}

interface PriceChartProps {
  data: PriceData[];
}

const mockData = [
  { name: 'Q1 \'26', value: 200 },
  { name: 'Q2 \'26', value: 400 },
  { name: 'Q3 \'26', value: 300 },
  { name: 'Q4 \'26', value: 280 },
];

export default function PriceChart({ data = mockData }: PriceChartProps) {
  return (
    <div className="bg-white p-6 rounded-lg shadow">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-semibold text-gray-700">Price Performance</h2>
      </div>
      <div className="h-80">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
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