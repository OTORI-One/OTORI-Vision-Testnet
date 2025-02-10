import { useState, useEffect } from 'react';
import MultiSigApproval from './MultiSigApproval';
import PositionManagement from './PositionManagement';
import TokenMinting from './TokenMinting';
import TransactionHistory from './TransactionHistory';
import { useOVTClient } from '../../src/hooks/useOVTClient';
import { isAdminWallet } from '../../src/utils/adminUtils';
import { useLaserEyes } from '@omnisat/lasereyes';

enum AdminView {
  POSITIONS = 'positions',
  MINT = 'mint',
  HISTORY = 'history',
}

export default function AdminDashboard() {
  const [activeView, setActiveView] = useState<AdminView>(AdminView.POSITIONS);
  const [isMultiSigModalOpen, setIsMultiSigModalOpen] = useState(false);
  const [pendingAction, setPendingAction] = useState<any>(null);
  const { isLoading, error } = useOVTClient();
  const { address } = useLaserEyes();

  // Check if current wallet is an admin
  if (!address || !isAdminWallet(address)) {
    return (
      <div className="bg-red-50 border-l-4 border-red-400 p-4">
        <div className="flex">
          <div className="ml-3">
            <p className="text-sm text-red-700">
              Access denied. This dashboard is only accessible to admin wallets.
            </p>
          </div>
        </div>
      </div>
    );
  }

  const handleActionRequiringMultiSig = (action: any) => {
    setPendingAction(action);
    setIsMultiSigModalOpen(true);
  };

  const handleMultiSigComplete = async (signatures: string[]) => {
    if (!pendingAction) return;
    
    try {
      // Execute the action with collected signatures
      await pendingAction.execute(signatures);
      setIsMultiSigModalOpen(false);
      setPendingAction(null);
    } catch (err) {
      console.error('Failed to execute action:', err);
    }
  };

  return (
    <div className="space-y-6">
      {/* Admin Navigation */}
      <div className="bg-white shadow rounded-lg p-4">
        <nav className="flex space-x-4">
          <button
            onClick={() => setActiveView(AdminView.POSITIONS)}
            className={`px-4 py-2 rounded-md ${
              activeView === AdminView.POSITIONS
                ? 'bg-blue-600 text-white'
                : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            Portfolio Positions
          </button>
          <button
            onClick={() => setActiveView(AdminView.MINT)}
            className={`px-4 py-2 rounded-md ${
              activeView === AdminView.MINT
                ? 'bg-blue-600 text-white'
                : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            Mint Tokens
          </button>
          <button
            onClick={() => setActiveView(AdminView.HISTORY)}
            className={`px-4 py-2 rounded-md ${
              activeView === AdminView.HISTORY
                ? 'bg-blue-600 text-white'
                : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            Transaction History
          </button>
        </nav>
      </div>

      {/* Main Content Area */}
      <div className="bg-white shadow rounded-lg p-6">
        {isLoading && (
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
        )}

        {error && (
          <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
            {error}
          </div>
        )}

        {!isLoading && !error && (
          <>
            {activeView === AdminView.POSITIONS && (
              <PositionManagement onActionRequiringMultiSig={handleActionRequiringMultiSig} />
            )}
            {activeView === AdminView.MINT && (
              <TokenMinting onActionRequiringMultiSig={handleActionRequiringMultiSig} />
            )}
            {activeView === AdminView.HISTORY && (
              <TransactionHistory />
            )}
          </>
        )}
      </div>

      {/* MultiSig Modal */}
      <MultiSigApproval
        isOpen={isMultiSigModalOpen}
        onClose={() => {
          setIsMultiSigModalOpen(false);
          setPendingAction(null);
        }}
        onComplete={handleMultiSigComplete}
        action={pendingAction}
      />
    </div>
  );
} 