import { useState } from 'react';
import { useOVTClient } from '../../src/hooks/useOVTClient';

const SATS_PER_BTC = 100000000;

interface PositionManagementProps {
  onActionRequiringMultiSig: (action: any) => void;
}

export default function PositionManagement({ onActionRequiringMultiSig }: PositionManagementProps) {
  const { addPosition, getPositions, formatValue } = useOVTClient();
  const [newPosition, setNewPosition] = useState({
    name: '',
    description: '',
    investmentAmount: '', // in BTC
    pricePerToken: '', // in sats
  });
  const [error, setError] = useState<string | null>(null);

  // Calculate token amount based on investment amount and price per token
  const calculateTokenAmount = (): number => {
    const btcAmount = parseFloat(newPosition.investmentAmount);
    const pricePerToken = parseFloat(newPosition.pricePerToken);
    
    if (isNaN(btcAmount) || isNaN(pricePerToken) || pricePerToken <= 0) {
      return 0;
    }

    // Convert BTC to sats and divide by price per token in sats
    return Math.floor((btcAmount * SATS_PER_BTC) / pricePerToken);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    try {
      const btcAmount = parseFloat(newPosition.investmentAmount);
      const pricePerToken = parseFloat(newPosition.pricePerToken);
      
      if (isNaN(btcAmount) || btcAmount <= 0) {
        throw new Error('Invalid investment amount');
      }
      if (isNaN(pricePerToken) || pricePerToken <= 0) {
        throw new Error('Invalid price per token');
      }

      const satsValue = Math.floor(btcAmount * SATS_PER_BTC);
      const tokenAmount = calculateTokenAmount();
      
      // Create new position
      const position = {
        name: newPosition.name,
        description: newPosition.description,
        value: satsValue,
        tokenAmount,
        pricePerToken: Math.floor(parseFloat(newPosition.pricePerToken)),
      };

      // This will require multi-sig approval
      onActionRequiringMultiSig({
        type: 'ADD_POSITION',
        description: `Add position for ${position.name}`,
        data: position,
        execute: async (signatures: string[]) => {
          await addPosition(position);
          // Reset form
          setNewPosition({
            name: '',
            description: '',
            investmentAmount: '',
            pricePerToken: ''
          });
        }
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add position');
    }
  };

  const positions = getPositions();

  return (
    <div className="space-y-6">
      <div className="bg-white shadow sm:rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium leading-6 text-gray-900">Add New Position</h3>
          
          <form onSubmit={handleSubmit} className="mt-5 space-y-4">
            <div>
              <label htmlFor="name" className="block text-sm font-medium text-gray-700">
                Project Name
              </label>
              <input
                type="text"
                name="name"
                id="name"
                required
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                value={newPosition.name}
                onChange={(e) => setNewPosition(prev => ({ ...prev, name: e.target.value }))}
              />
            </div>

            <div>
              <label htmlFor="description" className="block text-sm font-medium text-gray-700">
                Description
              </label>
              <textarea
                id="description"
                name="description"
                rows={3}
                required
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                value={newPosition.description}
                onChange={(e) => setNewPosition(prev => ({ ...prev, description: e.target.value }))}
              />
            </div>

            <div>
              <label htmlFor="investmentAmount" className="block text-sm font-medium text-gray-700">
                Investment Amount (BTC)
              </label>
              <div className="mt-1 relative rounded-md shadow-sm">
                <input
                  type="number"
                  name="investmentAmount"
                  id="investmentAmount"
                  required
                  step="0.00000001"
                  min="0"
                  className="block w-full rounded-md border-gray-300 pl-7 pr-12 focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                  placeholder="0.00000000"
                  value={newPosition.investmentAmount}
                  onChange={(e) => setNewPosition(prev => ({ ...prev, investmentAmount: e.target.value }))}
                />
                <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                  <span className="text-gray-500 sm:text-sm">BTC</span>
                </div>
              </div>
            </div>

            <div>
              <label htmlFor="pricePerToken" className="block text-sm font-medium text-gray-700">
                Price per Token (sats)
              </label>
              <div className="mt-1 relative rounded-md shadow-sm">
                <input
                  type="number"
                  name="pricePerToken"
                  id="pricePerToken"
                  required
                  step="1"
                  min="1"
                  className="block w-full rounded-md border-gray-300 pl-7 pr-12 focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                  placeholder="0"
                  value={newPosition.pricePerToken}
                  onChange={(e) => setNewPosition(prev => ({ ...prev, pricePerToken: e.target.value }))}
                />
                <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                  <span className="text-gray-500 sm:text-sm">sats</span>
                </div>
              </div>
            </div>

            <div>
              <label htmlFor="tokenAmount" className="block text-sm font-medium text-gray-700">
                Token Amount (Calculated)
              </label>
              <div className="mt-1 relative rounded-md shadow-sm">
                <input
                  type="text"
                  id="tokenAmount"
                  readOnly
                  className="block w-full rounded-md border-gray-300 bg-gray-50 pl-7 pr-12 focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
                  value={new Intl.NumberFormat().format(calculateTokenAmount())}
                />
                <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                  <span className="text-gray-500 sm:text-sm">tokens</span>
                </div>
              </div>
              <p className="mt-1 text-sm text-gray-500">
                Tokens to receive based on investment amount and price
              </p>
            </div>

            {error && (
              <div className="rounded-md bg-red-50 p-4">
                <div className="flex">
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-red-800">Error</h3>
                    <div className="mt-2 text-sm text-red-700">
                      <p>{error}</p>
                    </div>
                  </div>
                </div>
              </div>
            )}

            <button
              type="submit"
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Add Position
            </button>
          </form>
        </div>
      </div>

      {/* Current Positions */}
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900">Current Positions</h3>
        </div>
        <div className="border-t border-gray-200">
          <ul role="list" className="divide-y divide-gray-200">
            {positions.map((position, index) => (
              <li key={index} className="px-4 py-4 sm:px-6">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <h4 className="text-lg font-medium text-gray-900">{position.name}</h4>
                    <p className="mt-1 text-sm text-gray-500">{position.description}</p>
                    <p className="mt-1 text-sm text-gray-500">
                      {new Intl.NumberFormat().format(position.tokenAmount)} tokens @ {formatValue(position.pricePerToken, 'btc')}
                    </p>
                  </div>
                  <div className="ml-4 flex-shrink-0">
                    <div className="text-sm text-gray-900 text-right">
                      Initial: {formatValue(position.value, 'btc')}
                    </div>
                    <div className="text-sm text-gray-900 text-right">
                      Current: {formatValue(position.current, 'btc')}
                    </div>
                    <div className={`text-sm ${position.change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                      {position.change >= 0 ? '+' : ''}{position.change}%
                    </div>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
} 