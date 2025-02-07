import { useMemo } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface NAVData {
  name: string;
  initial: number;
  current: number;
}

interface NAVVisualizationProps {
  data: NAVData[];
  totalValue: string;
  changePercentage: string;
}

export default function NAVVisualization({ data, totalValue, changePercentage }: NAVVisualizationProps) {
  const formattedData = useMemo(() => {
    return data.map(item => ({
      ...item,
      initial: Number(item.initial.toFixed(2)),
      current: Number(item.current.toFixed(2)),
    }));
  }, [data]);

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
              left: 20,
              bottom: 5,
            }}
          >
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Bar dataKey="initial" stackId="a" fill="#93C5FD" name="Initial" />
            <Bar dataKey="current" stackId="a" fill="#3B82F6" name="Current" />
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
    </div>
  );
} 