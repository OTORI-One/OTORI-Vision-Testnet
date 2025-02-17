import { useState, useEffect } from 'react';
import { useOVTClient } from '../../src/hooks/useOVTClient';
import { ArrowUpIcon, ArrowDownIcon } from '@heroicons/react/24/outline';

interface Transaction {
  txid: string;
  type: 'mint' | 'burn' | 'transfer' | 'position_entry' | 'position_exit';
  amount: number;
  timestamp: number;
  status: 'pending' | 'confirmed' | 'failed';
  details: {
    reason?: string;
    position?: string;
    signatures?: string[];
    currency?: string;
  };
}

export default function TransactionHistory() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [filter, setFilter] = useState<string>('all');
  const { getTransactionHistory, isLoading, error, formatValue } = useOVTClient();

  useEffect(() => {
    const fetchTransactions = async () => {
      try {
        const history = await getTransactionHistory();
        setTransactions(history);
      } catch (err) {
        console.error('Failed to fetch transaction history:', err);
      }
    };

    fetchTransactions();
  }, [getTransactionHistory]);

  const filteredTransactions = transactions.filter(tx => 
    filter === 'all' || tx.type === filter
  );

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'confirmed': return 'text-green-600 bg-green-100';
      case 'pending': return 'text-yellow-600 bg-yellow-100';
      case 'failed': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'mint': return <ArrowDownIcon className="h-5 w-5 text-green-500" />;
      case 'burn': return <ArrowUpIcon className="h-5 w-5 text-red-500" />;
      default: return null;
    }
  };

  const formatAmount = (tx: Transaction) => {
    if (tx.type === 'position_entry') {
      // For position entries, format BTC value
      return formatValue(tx.amount, 'btc');
    }
    // For other transactions, show OVT amount
    return `${tx.amount.toLocaleString()} OVT`;
  };

  return (
    <div className="space-y-6">
      {/* Filter Controls */}
      <div className="flex justify-between items-center">
        <div className="flex space-x-4">
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
          >
            <option value="all">All Transactions</option>
            <option value="mint">Minting</option>
            <option value="burn">Burning</option>
            <option value="transfer">Transfers</option>
            <option value="position_entry">Position Entries</option>
            <option value="position_exit">Position Exits</option>
          </select>
        </div>
      </div>

      {/* Transaction List */}
      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        {isLoading ? (
          <div className="flex justify-center items-center py-12">
            <div data-testid="loading-spinner" className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
        ) : error ? (
          <div className="p-4 text-red-600 bg-red-50">{error}</div>
        ) : (
          <ul role="list" className="divide-y divide-gray-200">
            {filteredTransactions.map((tx) => (
              <li key={tx.txid}>
                <div className="px-4 py-4 sm:px-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center">
                      {getTypeIcon(tx.type)}
                      <p className="ml-2 text-sm font-medium text-gray-900">
                        {tx.type.charAt(0).toUpperCase() + tx.type.slice(1).replace('_', ' ')}
                      </p>
                      <span className={`ml-2 px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${getStatusColor(tx.status)}`}>
                        {tx.status}
                      </span>
                    </div>
                    <div className="ml-2 flex-shrink-0 flex">
                      <p className="text-sm text-gray-500">{formatDate(tx.timestamp)}</p>
                    </div>
                  </div>
                  <div className="mt-2 sm:flex sm:justify-between">
                    <div className="sm:flex">
                      <p className="flex items-center text-sm text-gray-500">
                        Amount: {formatAmount(tx)}
                      </p>
                      {tx.details.reason && (
                        <p className="mt-2 flex items-center text-sm text-gray-500 sm:mt-0 sm:ml-6">
                          Reason: {tx.details.reason}
                        </p>
                      )}
                    </div>
                    <div className="mt-2 flex items-center text-sm text-gray-500 sm:mt-0">
                      <a
                        href={`https://mempool.space/tx/${tx.txid}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-600 hover:text-blue-800"
                      >
                        View on Explorer
                      </a>
                    </div>
                  </div>
                  {tx.details.signatures && (
                    <div className="mt-2">
                      <p className="text-sm text-gray-500">
                        Signatures: {tx.details.signatures.length}/3
                      </p>
                    </div>
                  )}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
} 