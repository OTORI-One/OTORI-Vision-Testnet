import { useState } from 'react';
import Head from 'next/head';
import { ArrowUpIcon, ArrowDownIcon } from '@heroicons/react/24/outline';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

// Mock data for the NAV chart
const navData = [
  { name: 'Q1 \'26', value: 200 },
  { name: 'Q2 \'26', value: 400 },
  { name: 'Q3 \'26', value: 300 },
  { name: 'Q4 \'26', value: 280 },
];

export default function Dashboard() {
  const [isWalletConnected, setIsWalletConnected] = useState(false);
  const [ovtBalance, setOvtBalance] = useState('0');
  const [navValue, setNavValue] = useState('8.25M');
  const [navChange, setNavChange] = useState('+5%');

  const handleConnectWallet = async () => {
    try {
      // TODO: Implement actual Bitcoin wallet connection
      setIsWalletConnected(true);
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    }
  };

  return (
    <div className="min-h-screen bg-gray-100">
      <Head>
        <title>OTORI Vision Token Dashboard</title>
        <meta name="description" content="OTORI Vision Token Performance Dashboard" />
      </Head>

      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        {/* Header with Wallet Connection */}
        <div className="flex justify-between items-center mb-8">
          <div className="flex items-center">
            <img src="/logo.svg" alt="OTORI" className="h-8 w-8 mr-2" />
            <h1 className="text-2xl font-bold text-gray-900">OTORI Vision Token Performance</h1>
          </div>
          <button
            onClick={handleConnectWallet}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            {isWalletConnected ? 'Wallet Connected' : 'Connect Wallet'}
          </button>
        </div>

        {/* Main Dashboard Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* NAV Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Net Asset Value</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{navValue}</p>
              <p className="text-sm text-green-600 flex items-center">
                <ArrowUpIcon className="h-4 w-4 mr-1" />
                {navChange}
              </p>
            </div>
          </div>

          {/* Buy OVT Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Buy OVT</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">$288</p>
              <p className="text-sm text-gray-500">₿ 0.00</p>
            </div>
          </div>

          {/* Sell OVT Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Sell OVT</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">$288</p>
              <p className="text-sm text-gray-500">₿ 0.00</p>
            </div>
          </div>
        </div>

        {/* NAV Chart */}
        <div className="mt-8 bg-white p-6 rounded-lg shadow">
          <h2 className="text-lg font-semibold text-gray-700 mb-4">NAV Performance</h2>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={navData}>
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
      </main>
    </div>
  );
} 