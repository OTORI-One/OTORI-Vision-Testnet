import { useState, useEffect, useMemo } from 'react';
import Head from 'next/head';
import { ArrowUpIcon, CurrencyDollarIcon, CircleStackIcon } from '@heroicons/react/24/outline';
import WalletConnector from '../components/WalletConnector';
import NAVVisualization from '../components/NAVVisualization';
import PriceChart from '../components/PriceChart';
import ChartToggle from '../components/ChartToggle';
import { useOVTClient } from '../src/hooks/useOVTClient';
import AdminDashboard from '../components/admin/AdminDashboard';
import { useBitcoinPrice } from '../src/hooks/useBitcoinPrice';

// Constants for numeric handling
const SATS_PER_BTC = 100000000;

export default function Dashboard() {
  const [connectedAddress, setConnectedAddress] = useState<string | null>(null);
  const [activeChart, setActiveChart] = useState<'price' | 'nav'>('nav');
  const [buyAmount, setBuyAmount] = useState<string>('');
  const [sellAmount, setSellAmount] = useState<string>('');
  const [baseCurrency, setBaseCurrency] = useState<'usd' | 'btc'>('usd');
  const { isLoading, error, navData, formatValue, btcPrice } = useOVTClient();

  // Calculate OVT price based on NAV
  const ovtPrice = useMemo(() => {
    // Extract numeric value and convert to sats
    const btcValue = Number(navData.totalValue.replace(/[^0-9.]/g, ''));
    const satsValue = btcValue * SATS_PER_BTC;
    // Calculate price per OVT token (in sats)
    const pricePerOVT = Math.floor(satsValue / 1000000); // Assuming 1M total OVT supply
    return pricePerOVT;
  }, [navData.totalValue]);

  // Format NAV value according to selected currency
  const formattedNAV = useMemo(() => {
    const btcValue = Number(navData.totalValue.replace(/[^0-9.]/g, ''));
    const satsValue = btcValue * SATS_PER_BTC;
    return formatValue(satsValue, baseCurrency);
  }, [navData.totalValue, baseCurrency, formatValue]);

  // Format currency values
  const formatCurrency = (value: number) => {
    if (baseCurrency === 'usd') {
      const currentBtcPrice = btcPrice || 40000; // Use real BTC price if available
      const usdValue = (value / SATS_PER_BTC) * currentBtcPrice;
      return formatValue(usdValue, baseCurrency);
    } else {
      return formatValue(value, baseCurrency);
    }
  };

  const handleConnectWallet = (address: string) => {
    setConnectedAddress(address);
  };

  const handleDisconnectWallet = () => {
    setConnectedAddress(null);
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
                      className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                      min="0"
                      step="0.01"
                    />
                    <span className="text-sm text-gray-500">OVT</span>
                  </div>
                  <button 
                    onClick={() => {}}
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
                      className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                      min="0"
                      step="0.01"
                    />
                    <span className="text-sm text-gray-500">OVT</span>
                  </div>
                  <button 
                    onClick={() => {}}
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