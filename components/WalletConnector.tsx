import { useState, useCallback } from 'react';

interface WalletConnectorProps {
  onConnect: (address: string) => void;
  onDisconnect: () => void;
}

export default function WalletConnector({ onConnect, onDisconnect }: WalletConnectorProps) {
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connectWallet = useCallback(async () => {
    setIsConnecting(true);
    setError(null);

    try {
      // Check if window.bitcoin exists (Xverse wallet)
      if (typeof window !== 'undefined' && (window as any).bitcoin) {
        const bitcoin = (window as any).bitcoin;
        
        try {
          // Request connection to the wallet
          const accounts = await bitcoin.request({ method: 'requestAccounts' });
          
          if (accounts && accounts.length > 0) {
            onConnect(accounts[0]);
          } else {
            throw new Error('No accounts found');
          }
        } catch (err: any) {
          console.error('Wallet connection error:', err);
          setError(err.message || 'Failed to connect wallet');
        }
      } else {
        setError('Please install Xverse wallet');
      }
    } catch (err: any) {
      console.error('Wallet connection error:', err);
      setError(err.message || 'Failed to connect wallet');
    } finally {
      setIsConnecting(false);
    }
  }, [onConnect]);

  const disconnectWallet = useCallback(() => {
    onDisconnect();
    setError(null);
  }, [onDisconnect]);

  return (
    <div>
      <button
        onClick={connectWallet}
        disabled={isConnecting}
        className="btn-primary"
      >
        {isConnecting ? 'Connecting...' : 'Connect Bitcoin Wallet'}
      </button>
      
      {error && (
        <p className="mt-2 text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
} 