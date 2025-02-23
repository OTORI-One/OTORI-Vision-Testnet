import { useState, useEffect, useMemo } from 'react';
import Head from 'next/head';
import { ArrowUpIcon, CurrencyDollarIcon, CircleStackIcon } from '@heroicons/react/24/outline';
import WalletConnector from '../components/WalletConnector';
import NAVVisualization from '../components/NAVVisualization';
import PriceChart from '../components/PriceChart';
import ChartToggle from '../components/ChartToggle';
import { useOVTClient, SATS_PER_BTC } from '../src/hooks/useOVTClient';
import AdminDashboard from '../components/admin/AdminDashboard';
import { useBitcoinPrice } from '../src/hooks/useBitcoinPrice';
import { useLaserEyes } from '@omnisat/lasereyes';

export default function Dashboard() {
  const [connectedAddress, setConnectedAddress] = useState<string | null>(null);
  const [activeChart, setActiveChart] = useState<'price' | 'nav'>('nav');
  const [buyAmount, setBuyAmount] = useState<string>('');
  const [sellAmount, setSellAmount] = useState<string>('');
  const [networkError, setNetworkError] = useState<string | null>(null);
  
  const { 
    isLoading, 
    error, 
    navData, 
    formatValue,
    baseCurrency,
    setBaseCurrency
  } = useOVTClient();
  const { price: btcPrice } = useBitcoinPrice();
  const { network } = useLaserEyes();

  // Calculate OVT price based on NAV
  const ovtPrice = useMemo(() => {
    const btcValue = Number(navData.totalValue.replace(/[^0-9.]/g, ''));
    const satsValue = btcValue * SATS_PER_BTC;
    const pricePerOVT = Math.floor(satsValue / 1000000); // Assuming 1M total OVT supply
    return pricePerOVT;
  }, [navData.totalValue]);

  // Use the NAV value directly from navData
  const formattedNAV = navData.totalValue;

  // Format currency values
  const formatCurrency = (value: number) => {
    return formatValue(value, baseCurrency);
  };

  const handleConnectWallet = (address: string) => {
    // Only set the connected address if we have one
    if (address) {
      setConnectedAddress(address);
    }
  };

  const handleDisconnectWallet = () => {
    setConnectedAddress(null);
    setNetworkError(null);
  };

  return (
    <div className="min-h-screen bg-gray-100">
      <Head>
        <title>OTORI Vision Token Dashboard</title>
        <meta name="description" content="OTORI Vision Token Dashboard" />
      </Head>

      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        {/* Header with Wallet Connection and Currency Toggle */}
        <div className="flex justify-between items-center mb-8">
          <div className="flex items-center">
            <img src="/logo.svg" alt="OTORI" className="h-8 w-8 mr-2" />
            <h1 className="text-2xl font-bold text-gray-900">OTORI Vision Token Dashboard</h1>
          </div>
          <div className="flex items-center space-x-4">
            <button
              onClick={() => setBaseCurrency(prev => prev === 'usd' ? 'btc' : 'usd')}
              className="p-2 rounded-lg hover:bg-gray-200 transition-colors flex items-center space-x-1"
            >
              {baseCurrency === 'usd' ? (
                <>
                  <CurrencyDollarIcon className="h-6 w-6" />
                  <span className="text-sm">USD</span>
                </>
              ) : (
                <>
                  <CircleStackIcon className="h-6 w-6" />
                  <span className="text-sm">BTC</span>
                </>
              )}
            </button>
            <WalletConnector 
              onConnect={handleConnectWallet}
              onDisconnect={handleDisconnectWallet}
              connectedAddress={connectedAddress || undefined}
            />
          </div>
        </div>

        {/* Admin Dashboard (only shown to connected admin wallets) */}
        {connectedAddress && (
          <div className="mb-8">
            <AdminDashboard />
          </div>
        )}

        {/* Main Dashboard Grid */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          {/* NAV Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Net Asset Value</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{formattedNAV}</p>
              <p className="text-sm text-green-600 flex items-center">
                <ArrowUpIcon className="h-4 w-4 mr-1" />
                {navData.changePercentage}
              </p>
            </div>
          </div>

          {/* Initial Investment Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Your Investment</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{formatCurrency(0)}</p>
              <p className="text-sm text-gray-500">0 OVT</p>
            </div>
          </div>

          {/* Buy OVT Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Buy OVT</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{formatCurrency(ovtPrice)}</p>
              <p className="text-sm text-gray-500">per OVT</p>
              {connectedAddress && (
                <div className="mt-4 space-y-4">
                  <div className="flex items-center space-x-2">
                    <input
                      type="number"
                      value={buyAmount}
                      onChange={(e) => setBuyAmount(e.target.value)}
                      placeholder="Enter amount"
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                      min="0"
                      step="1"
                    />
                  </div>
                  <button 
                    disabled={isLoading || !buyAmount}
                    className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
                  >
                    {isLoading ? 'Processing...' : 'Buy OVT'}
                  </button>
                </div>
              )}
            </div>
          </div>

          {/* Sell OVT Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Sell OVT</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{formatCurrency(ovtPrice)}</p>
              <p className="text-sm text-gray-500">per OVT</p>
              {connectedAddress && (
                <div className="mt-4 space-y-4">
                  <div className="flex items-center space-x-2">
                    <input
                      type="number"
                      value={sellAmount}
                      onChange={(e) => setSellAmount(e.target.value)}
                      placeholder="Enter amount"
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                      min="0"
                      step="1"
                    />
                  </div>
                  <button 
                    disabled={isLoading || !sellAmount}
                    className="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50"
                  >
                    {isLoading ? 'Processing...' : 'Sell OVT'}
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Chart Section */}
        <div className="mt-8">
          <ChartToggle activeChart={activeChart} onToggle={setActiveChart} />
          
          <div className="mt-4">
            {activeChart === 'price' ? (
              <PriceChart 
                data={[
                  { name: 'Q1 \'26', value: 200 },
                  { name: 'Q2 \'26', value: 400 },
                  { name: 'Q3 \'26', value: 300 },
                  { name: 'Q4 \'26', value: 280 },
                ]} 
                baseCurrency={baseCurrency}
              />
            ) : (
              <NAVVisualization 
                data={navData.portfolioItems}
                totalValue={navData.totalValue}
                changePercentage={navData.changePercentage}
                baseCurrency={baseCurrency}
              />
            )}
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