import { useState, useEffect } from 'react';
import Head from 'next/head';
import { ArrowUpIcon } from '@heroicons/react/24/outline';
import WalletConnector from '../components/WalletConnector';
import NAVVisualization from '../components/NAVVisualization';
import Portfolio from '../components/Portfolio';
import PriceChart from '../components/PriceChart';
import ChartToggle from '../components/ChartToggle';
import { useOVTClient } from '../src/hooks/useOVTClient';

export default function Dashboard() {
  const [connectedAddress, setConnectedAddress] = useState<string | null>(null);
  const [activeChart, setActiveChart] = useState<'price' | 'nav'>('price');
  const { isLoading, error, navData, buyOVT, sellOVT } = useOVTClient();

  const handleConnectWallet = (address: string) => {
    setConnectedAddress(address);
  };

  const handleDisconnectWallet = () => {
    setConnectedAddress(null);
  };

  const handleBuyOVT = async () => {
    if (!connectedAddress) return;
    try {
      await buyOVT(100); // Example amount
    } catch (err) {
      console.error('Failed to buy OVT:', err);
    }
  };

  const handleSellOVT = async () => {
    if (!connectedAddress) return;
    try {
      await sellOVT(100); // Example amount
    } catch (err) {
      console.error('Failed to sell OVT:', err);
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
          <WalletConnector 
            onConnect={handleConnectWallet}
            onDisconnect={handleDisconnectWallet}
            connectedAddress={connectedAddress || undefined}
          />
        </div>

        {/* Main Dashboard Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* NAV Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Net Asset Value</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{navData.totalValue}</p>
              <p className="text-sm text-green-600 flex items-center">
                <ArrowUpIcon className="h-4 w-4 mr-1" />
                {navData.changePercentage}
              </p>
            </div>
          </div>

          {/* Buy OVT Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Buy OVT</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">$288</p>
              <p className="text-sm text-gray-500">₿ 0.00</p>
              {connectedAddress && (
                <button 
                  onClick={handleBuyOVT}
                  disabled={isLoading}
                  className="mt-4 w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
                >
                  {isLoading ? 'Processing...' : 'Buy OVT'}
                </button>
              )}
            </div>
          </div>

          {/* Sell OVT Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Sell OVT</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">$288</p>
              <p className="text-sm text-gray-500">₿ 0.00</p>
              {connectedAddress && (
                <button 
                  onClick={handleSellOVT}
                  disabled={isLoading}
                  className="mt-4 w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50"
                >
                  {isLoading ? 'Processing...' : 'Sell OVT'}
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Chart Toggle and Charts Section */}
        <div className="mt-8">
          <ChartToggle activeChart={activeChart} onToggle={setActiveChart} />
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="md:col-span-1">
              {activeChart === 'price' ? (
                <PriceChart data={[
                  { name: 'Q1 \'26', value: 200 },
                  { name: 'Q2 \'26', value: 400 },
                  { name: 'Q3 \'26', value: 300 },
                  { name: 'Q4 \'26', value: 280 },
                ]} />
              ) : (
                <NAVVisualization 
                  data={navData.portfolioItems.map(item => ({
                    name: item.name,
                    initial: item.value * 0.7,
                    current: item.value,
                  }))}
                  totalValue={navData.totalValue}
                  changePercentage={navData.changePercentage}
                />
              )}
            </div>

            {/* Portfolio */}
            <Portfolio 
              items={navData.portfolioItems}
              initialInvestment="₿ 0.07 = $ 7200"
              isWalletConnected={!!connectedAddress}
            />
          </div>
        </div>

        {error && (
          <div className="mt-4 p-4 bg-red-100 text-red-700 rounded-lg">
            {error}
          </div>
        )}
      </main>
    </div>
  );
} 