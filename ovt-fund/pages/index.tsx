import { useState, useEffect, useMemo } from 'react';
import Head from 'next/head';
import { ArrowUpIcon, CurrencyDollarIcon, CircleStackIcon } from '@heroicons/react/24/outline';
import WalletConnector from '../components/WalletConnector';
import NAVVisualization from '../components/NAVVisualization';
import Portfolio from '../components/Portfolio';
import PriceChart from '../components/PriceChart';
import ChartToggle from '../components/ChartToggle';
import { useOVTClient } from '../src/hooks/useOVTClient';

export default function Dashboard() {
  const [connectedAddress, setConnectedAddress] = useState<string | null>(null);
  const [activeChart, setActiveChart] = useState<'price' | 'nav'>('nav');
  const [buyAmount, setBuyAmount] = useState<string>('');
  const [sellAmount, setSellAmount] = useState<string>('');
  const [baseCurrency, setBaseCurrency] = useState<'usd' | 'btc'>('usd');
  const { isLoading, error, navData, buyOVT, sellOVT } = useOVTClient();

  // Calculate OVT price based on NAV
  const ovtPrice = useMemo(() => {
    const totalValue = parseFloat(navData.totalValue.replace(/[^0-9.]/g, '')) * 1000000;
    const pricePerOVT = totalValue / 1000000; // Assuming 1M total OVT supply
    return pricePerOVT;
  }, [navData.totalValue]);

  // Format currency values
  const formatCurrency = (value: number) => {
    if (baseCurrency === 'usd') {
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

  const handleConnectWallet = (address: string) => {
    setConnectedAddress(address);
  };

  const handleDisconnectWallet = () => {
    setConnectedAddress(null);
  };

  const handleBuyOVT = async () => {
    if (!connectedAddress || !buyAmount) return;
    try {
      const amount = parseFloat(buyAmount);
      if (isNaN(amount) || amount <= 0) {
        throw new Error('Invalid amount');
      }
      await buyOVT(amount);
      setBuyAmount('');
    } catch (err) {
      console.error('Failed to buy OVT:', err);
    }
  };

  const handleSellOVT = async () => {
    if (!connectedAddress || !sellAmount) return;
    try {
      const amount = parseFloat(sellAmount);
      if (isNaN(amount) || amount <= 0) {
        throw new Error('Invalid amount');
      }
      await sellOVT(amount);
      setSellAmount('');
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
        {/* Header with Wallet Connection and Currency Toggle */}
        <div className="flex justify-between items-center mb-8">
          <div className="flex items-center">
            <img src="/logo.svg" alt="OTORI" className="h-8 w-8 mr-2" />
            <h1 className="text-2xl font-bold text-gray-900">OTORI Vision Token Performance</h1>
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

        {/* Main Dashboard Grid */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          {/* NAV Card */}
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-lg font-semibold text-gray-700">Net Asset Value</h2>
            <div className="mt-2">
              <p className="text-3xl font-bold">{formatCurrency(parseFloat(navData.totalValue.replace(/[^0-9.]/g, '')) * 1000000)}</p>
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
                    onClick={handleBuyOVT}
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
                    onClick={handleSellOVT}
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
                data={navData.portfolioItems.map(item => ({
                  name: item.name,
                  initial: item.value,
                  current: item.current,
                  change: item.change,
                  description: item.description
                }))}
                totalValue={formatCurrency(parseFloat(navData.totalValue.replace(/[^0-9.]/g, '')) * 1000000)}
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