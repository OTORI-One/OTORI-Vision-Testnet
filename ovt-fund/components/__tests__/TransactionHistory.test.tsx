import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import TransactionHistory from '../admin/TransactionHistory';
import { ArrowDownIcon, ArrowUpIcon } from '@heroicons/react/24/outline';

// Mock heroicons
jest.mock('@heroicons/react/24/outline', () => ({
  ArrowDownIcon: () => <div data-testid="arrow-down-icon" />,
  ArrowUpIcon: () => <div data-testid="arrow-up-icon" />
}));

// Mock useOVTClient hook
const mockGetTransactionHistory = jest.fn();
const mockUseOVTClient: {
  getTransactionHistory: jest.Mock;
  isLoading: boolean;
  error: string | null;
} = {
  getTransactionHistory: mockGetTransactionHistory,
  isLoading: false,
  error: null
};

jest.mock('../../src/hooks/useOVTClient', () => ({
  useOVTClient: () => mockUseOVTClient
}));

describe('TransactionHistory', () => {
  const mockTransactions = [
    {
      txid: 'tx1',
      type: 'mint',
      amount: 1000,
      timestamp: 1677649200000, // 2023-03-01
      status: 'confirmed',
      details: {
        reason: 'Initial mint',
        signatures: ['sig1', 'sig2', 'sig3']
      }
    },
    {
      txid: 'tx2',
      type: 'position_entry',
      amount: 0.5,
      timestamp: 1677735600000, // 2023-03-02
      status: 'pending',
      details: {
        position: 'Project X',
        currency: 'BTC'
      }
    }
  ];

  beforeEach(async () => {
    jest.clearAllMocks();
    mockGetTransactionHistory.mockResolvedValue(mockTransactions);
    mockUseOVTClient.isLoading = false;
    mockUseOVTClient.error = null;
  });

  it('renders transaction list correctly', async () => {
    await act(async () => {
      render(<TransactionHistory />);
    });
    
    // Wait for transactions to load and check items
    await waitFor(() => {
      // Check for the reason text
      expect(screen.getByText(/Initial mint/i)).toBeInTheDocument();
      // Check for the amount text (including the label)
      expect(screen.getByText(/Amount: ₿0.5/)).toBeInTheDocument();
      // Check for signatures text
      expect(screen.getByText(/Signatures: 3\/3/)).toBeInTheDocument();
    });
  });

  it('handles loading state', async () => {
    mockUseOVTClient.isLoading = true;
    
    render(<TransactionHistory />);
    
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('handles error state', async () => {
    mockUseOVTClient.error = 'Failed to fetch transactions';
    
    render(<TransactionHistory />);
    
    expect(screen.getByText('Failed to fetch transactions')).toBeInTheDocument();
  });

  it('filters transactions correctly', async () => {
    await act(async () => {
      render(<TransactionHistory />);
    });
    
    // Wait for initial render
    await waitFor(() => {
      expect(screen.getByText(/Initial mint/i)).toBeInTheDocument();
    });
    
    // Filter by mint transactions
    await act(async () => {
      fireEvent.change(screen.getByRole('combobox'), { target: { value: 'mint' } });
    });
    
    // Should show mint transaction but not position entry
    expect(screen.getByText(/Initial mint/i)).toBeInTheDocument();
    expect(screen.queryByText(/Amount: ₿0.5/)).not.toBeInTheDocument();
  });

  it('formats different transaction types correctly', async () => {
    await act(async () => {
      render(<TransactionHistory />);
    });
    
    // Wait for transactions to load
    await waitFor(() => {
      // Check mint transaction amount
      expect(screen.getByText(/Amount: 1,000 OVT/)).toBeInTheDocument();
      // Check position entry amount
      expect(screen.getByText(/Amount: ₿0.5/)).toBeInTheDocument();
    });
  });

  it('displays correct status indicators', async () => {
    await act(async () => {
      render(<TransactionHistory />);
    });
    
    // Wait for transactions to load
    await waitFor(() => {
      expect(screen.getByText('confirmed')).toBeInTheDocument();
    });
    
    const confirmedStatus = screen.getByText('confirmed');
    const pendingStatus = screen.getByText('pending');
    
    expect(confirmedStatus).toHaveClass('text-green-600');
    expect(pendingStatus).toHaveClass('text-yellow-600');
  });
}); 