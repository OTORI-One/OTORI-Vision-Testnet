import { useState, useCallback } from 'react';
import { LEATHER, UNISAT, XVERSE, useLaserEyes, ProviderType } from '@omnisat/lasereyes';

interface WalletConnectorProps {
  onConnect: (address: string) => void;
  onDisconnect: () => void;
  connectedAddress?: string;
}

export default function WalletConnector({ onConnect, onDisconnect, connectedAddress }: WalletConnectorProps) {
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { connect, disconnect, address } = useLaserEyes();

  const handleConnect = useCallback(async (wallet: ProviderType) => {
    setIsConnecting(true);
    setError(null);

    try {
      await connect(wallet);
      if (address) {
        onConnect(address);
      }
    } catch (err: any) {
      console.error('Wallet connection error:', err);
      setError(err.message || 'Failed to connect wallet');
    } finally {
      setIsConnecting(false);
    }
  }, [connect, address, onConnect]);

  const handleDisconnect = useCallback(() => {
    disconnect();
    onDisconnect();
    setError(null);
  }, [disconnect, onDisconnect]);

  if (connectedAddress) {
    return (
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-green-500 rounded-full"></div>
          <span className="text-sm text-gray-600">
            {connectedAddress.slice(0, 6)}...{connectedAddress.slice(-4)}
          </span>
        </div>
        <button
          onClick={handleDisconnect}
          className="text-sm text-gray-500 hover:text-gray-700"
        >
          Disconnect
        </button>
      </div>
    );
  }

  return (
    <div className="flex items-center space-x-2">
      <button
        onClick={() => handleConnect(XVERSE)}
        disabled={isConnecting}
        className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
      >
        {isConnecting ? 'Connecting...' : 'Connect Xverse'}
      </button>
      
      <button
        onClick={() => handleConnect(UNISAT)}
        disabled={isConnecting}
        className="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors disabled:opacity-50"
      >
        {isConnecting ? 'Connecting...' : 'Connect Unisat'}
      </button>
      
      <button
        onClick={() => handleConnect(LEATHER)}
        disabled={isConnecting}
        className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors disabled:opacity-50"
      >
        {isConnecting ? 'Connecting...' : 'Connect Leather'}
      </button>
      
      {error && (
        <p className="mt-2 text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
} 