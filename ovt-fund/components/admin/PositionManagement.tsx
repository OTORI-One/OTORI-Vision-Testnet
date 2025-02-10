import { useState } from 'react';
import { useOVTClient } from '../../src/hooks/useOVTClient';

interface PositionManagementProps {
  onActionRequiringMultiSig: (action: any) => void;
}

interface PositionFormData {
  name: string;
  amount: string;
  pricePerToken: string;
  currencySpent: string;
  currency: 'BTC' | 'USD';
  transactionId?: string;
  safeInscriptionId?: string;
}

type PositionType = 'pre-tge' | 'post-tge';

export default function PositionManagement({ onActionRequiringMultiSig }: PositionManagementProps) {
  const [activeTab, setActiveTab] = useState<PositionType>('post-tge');
  const [formData, setFormData] = useState<PositionFormData>({
    name: '',
    amount: '',
    pricePerToken: '',
    currencySpent: '',
    currency: 'BTC',
  });
  const { isLoading } = useOVTClient();

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  const handleCurrencyChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setFormData(prev => ({ ...prev, currency: e.target.value as 'BTC' | 'USD' }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const action = {
      type: activeTab === 'pre-tge' ? 'ADD_PRE_TGE_POSITION' : 'ADD_POST_TGE_POSITION',
      description: `Add ${activeTab === 'pre-tge' ? 'Pre' : 'Post'}-TGE position for ${formData.name}`,
      data: {
        ...formData,
        amount: parseFloat(formData.amount),
        pricePerToken: parseFloat(formData.pricePerToken),
        currencySpent: parseFloat(formData.currencySpent),
      },
    };

    onActionRequiringMultiSig(action);
  };

  return (
    <div className="space-y-6">
      {/* Tab Navigation */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setActiveTab('post-tge')}
            className={`py-4 px-1 ${
              activeTab === 'post-tge'
                ? 'border-b-2 border-blue-500 text-blue-600'
                : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Post-TGE Positions
          </button>
          <button
            onClick={() => setActiveTab('pre-tge')}
            className={`py-4 px-1 ${
              activeTab === 'pre-tge'
                ? 'border-b-2 border-blue-500 text-blue-600'
                : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Pre-TGE Positions
          </button>
        </nav>
      </div>

      {/* Position Form */}
      <form onSubmit={handleSubmit} className="space-y-6">
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-700">
              Project Name
            </label>
            <input
              type="text"
              name="name"
              id="name"
              required
              value={formData.name}
              onChange={handleInputChange}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
            />
          </div>

          <div>
            <label htmlFor="amount" className="block text-sm font-medium text-gray-700">
              Investment Amount
            </label>
            <div className="mt-1 flex rounded-md shadow-sm">
              <input
                type="number"
                name="amount"
                id="amount"
                required
                min="0"
                step={formData.currency === 'BTC' ? '0.00000001' : '1'}
                value={formData.amount}
                onChange={handleInputChange}
                className="flex-1 rounded-l-md border-gray-300 focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
              />
              <select
                value={formData.currency}
                onChange={handleCurrencyChange}
                className="rounded-r-md border-l-0 border-gray-300 bg-gray-50 px-3 text-gray-500 sm:text-sm"
              >
                <option value="BTC">BTC</option>
                <option value="USD">USD</option>
              </select>
            </div>
          </div>

          <div>
            <label htmlFor="pricePerToken" className="block text-sm font-medium text-gray-700">
              Price per Token (sats)
            </label>
            <input
              type="number"
              name="pricePerToken"
              id="pricePerToken"
              required
              min="0"
              step="1"
              value={formData.pricePerToken}
              onChange={handleInputChange}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
            />
          </div>

          <div>
            <label htmlFor="currencySpent" className="block text-sm font-medium text-gray-700">
              Total Currency Spent (sats)
            </label>
            <input
              type="number"
              name="currencySpent"
              id="currencySpent"
              required
              min="0"
              step="1"
              value={formData.currencySpent}
              onChange={handleInputChange}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
            />
          </div>

          {activeTab === 'post-tge' && (
            <div className="sm:col-span-2">
              <label htmlFor="transactionId" className="block text-sm font-medium text-gray-700">
                Transaction ID
              </label>
              <input
                type="text"
                name="transactionId"
                id="transactionId"
                required
                value={formData.transactionId || ''}
                onChange={handleInputChange}
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
              />
            </div>
          )}

          {activeTab === 'pre-tge' && (
            <div className="sm:col-span-2">
              <label htmlFor="safeInscriptionId" className="block text-sm font-medium text-gray-700">
                SAFE/SAFT Inscription ID
              </label>
              <input
                type="text"
                name="safeInscriptionId"
                id="safeInscriptionId"
                required
                value={formData.safeInscriptionId || ''}
                onChange={handleInputChange}
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
              />
            </div>
          )}
        </div>

        <div className="flex justify-end">
          <button
            type="submit"
            disabled={isLoading}
            className="inline-flex justify-center rounded-md border border-transparent bg-blue-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50"
          >
            {isLoading ? 'Processing...' : 'Add Position'}
          </button>
        </div>
      </form>
    </div>
  );
} 