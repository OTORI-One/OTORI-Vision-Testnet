import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import PositionManagement from '../admin/PositionManagement';
import { useOVTClient } from '../../src/hooks/useOVTClient';
import { ArchClient } from '../../src/lib/archClient';

// Mock ArchClient instead of the hook
jest.mock('../../src/lib/archClient', () => {
  return {
    ArchClient: jest.fn().mockImplementation(() => ({
      getCurrentNAV: jest.fn().mockResolvedValue({
        value: 3375000000, // ₿33.75
        portfolioItems: [
          {
            name: 'Polymorphic Labs',
            value: 500000000,
            change: 420,
          },
          {
            name: 'VoltFi',
            value: 150000000,
            change: 250,
          }
        ]
      }),
      getTransactionHistory: jest.fn().mockResolvedValue([]),
      buyOVT: jest.fn().mockResolvedValue({ success: true }),
      sellOVT: jest.fn().mockResolvedValue({ success: true })
    }))
  };
});

describe('PositionManagement', () => {
  const mockOnActionRequiringMultiSig = jest.fn();

  const defaultProps = {
    onActionRequiringMultiSig: mockOnActionRequiringMultiSig,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the position management form with post-tge tab active by default', () => {
    render(<PositionManagement {...defaultProps} />);
    
    // Check if both tabs are present
    expect(screen.getByText('Post-TGE Positions')).toHaveClass('border-blue-500');
    expect(screen.getByText('Pre-TGE Positions')).not.toHaveClass('border-blue-500');
    
    // Check if form fields are present
    expect(screen.getByLabelText('Project Name')).toBeInTheDocument();
    expect(screen.getByLabelText('Investment Amount')).toBeInTheDocument();
    expect(screen.getByLabelText('Price per Token (sats)')).toBeInTheDocument();
    expect(screen.getByLabelText('Total Currency Spent')).toBeInTheDocument();
    expect(screen.getByLabelText('Transaction ID')).toBeInTheDocument();
    
    // Pre-TGE specific field should not be visible
    expect(screen.queryByLabelText('SAFE/SAFT Inscription ID')).not.toBeInTheDocument();
  });

  it('switches between pre-tge and post-tge tabs', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    // Switch to pre-tge tab
    const preTgeTab = screen.getByText('Pre-TGE Positions');
    await act(async () => {
      fireEvent.click(preTgeTab);
    });
    
    // Check if pre-tge specific field is visible
    expect(screen.getByLabelText('SAFE/SAFT Inscription ID')).toBeInTheDocument();
    expect(screen.queryByLabelText('Transaction ID')).not.toBeInTheDocument();
    
    // Switch back to post-tge tab
    const postTgeTab = screen.getByText('Post-TGE Positions');
    await act(async () => {
      fireEvent.click(postTgeTab);
    });
    
    // Check if post-tge specific field is visible
    expect(screen.getByLabelText('Transaction ID')).toBeInTheDocument();
    expect(screen.queryByLabelText('SAFE/SAFT Inscription ID')).not.toBeInTheDocument();
  });

  it('handles form input changes and validates currency spent calculations', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    const projectNameInput = screen.getByLabelText('Project Name');
    const amountInput = screen.getByLabelText('Investment Amount');
    const priceInput = screen.getByLabelText('Price per Token (sats)');
    const currencySpentInput = screen.getByLabelText('Total Currency Spent');
    const txIdInput = screen.getByLabelText('Transaction ID');
    
    await act(async () => {
      fireEvent.change(projectNameInput, { target: { value: 'Test Project' } });
      fireEvent.change(amountInput, { target: { value: '1000' } });
      fireEvent.change(priceInput, { target: { value: '100' } });
      fireEvent.change(txIdInput, { target: { value: 'tx123' } });
    });

    // Wait for all state updates to complete
    await waitFor(() => {
      expect(projectNameInput).toHaveValue('Test Project');
      // Expect string values for numeric inputs to match form state
      expect(amountInput).toHaveValue('1000');
      expect(priceInput).toHaveValue('100');
      expect(currencySpentInput).toHaveValue('100000');
      expect(txIdInput).toHaveValue('tx123');
      // Check formatted display
      expect(screen.getByText('100,000 sats')).toBeInTheDocument();
    });
  });

  it('handles currency selection and updates calculations', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    const currencySelect = screen.getByRole('combobox');
    const amountInput = screen.getByLabelText('Investment Amount');
    const priceInput = screen.getByLabelText('Price per Token (sats)');
    const currencySpentInput = screen.getByLabelText('Total Currency Spent');
    
    // Test BTC calculations
    await act(async () => {
      fireEvent.change(currencySelect, { target: { value: 'BTC' } });
      fireEvent.change(amountInput, { target: { value: '1000' } });
      fireEvent.change(priceInput, { target: { value: '100' } });
    });
    
    await waitFor(() => {
      expect(currencySpentInput).toHaveValue('100000');
      expect(screen.getByText('100,000 sats')).toBeInTheDocument();
    });
    
    // Test USD calculations (using mocked Bitcoin price of 40000 from jest.setup.js)
    await act(async () => {
      fireEvent.change(currencySelect, { target: { value: 'USD' } });
      fireEvent.change(amountInput, { target: { value: '1000' } });
      fireEvent.change(priceInput, { target: { value: '1' } }); // $1 per token
    });
    
    await waitFor(() => {
      expect(currencySpentInput).toHaveValue('2500000');
      expect(screen.getByText('2,500,000 sats')).toBeInTheDocument();
    });
  });

  it('submits post-tge position form with proper data processing', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    // Fill out the form
    await act(async () => {
      fireEvent.change(screen.getByLabelText('Project Name'), { target: { value: 'Test Project' } });
      fireEvent.change(screen.getByLabelText('Investment Amount'), { target: { value: '1000' } });
      fireEvent.change(screen.getByLabelText('Price per Token (sats)'), { target: { value: '100' } });
      fireEvent.change(screen.getByLabelText('Transaction ID'), { target: { value: 'tx123' } });
    });
    
    // Submit the form
    const submitButton = screen.getByText('Add Position');
    await act(async () => {
      fireEvent.click(submitButton);
    });
    
    // Wait for validation and submission
    await waitFor(() => {
      expect(mockOnActionRequiringMultiSig).toHaveBeenCalledWith({
        type: 'ADD_POST_TGE_POSITION',
        description: 'Add Post-TGE position for Test Project',
        data: {
          name: 'Test Project',
          amount: 1000,
          pricePerToken: 100,
          currencySpent: 100000,
          currency: 'BTC',
          transactionId: 'tx123'
        }
      });
    });
  });

  it('validates required fields with delay and shows appropriate error messages', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    // Try to submit without filling required fields
    const submitButton = screen.getByText('Add Position');
    await act(async () => {
      fireEvent.click(submitButton);
    });
    
    // Wait for validation messages (due to delay)
    await waitFor(() => {
      expect(screen.getByText('Project Name is required')).toBeInTheDocument();
      expect(screen.getByText('Investment Amount must be greater than 0')).toBeInTheDocument();
      expect(screen.getByText('Price per Token must be greater than 0')).toBeInTheDocument();
      expect(screen.getByText('Transaction ID is required')).toBeInTheDocument();
    }, { timeout: 1000 }); // Increased timeout to account for validation delay
    
    // Check that the multisig action was not triggered
    expect(mockOnActionRequiringMultiSig).not.toHaveBeenCalled();
  });

  it('clears validation errors when user starts typing', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    // Submit empty form to trigger validation errors
    const submitButton = screen.getByText('Add Position');
    await act(async () => {
      fireEvent.click(submitButton);
    });
    
    // Wait for validation messages
    await waitFor(() => {
      expect(screen.getByText('Project Name is required')).toBeInTheDocument();
    });
    
    // Start typing in project name
    await act(async () => {
      fireEvent.change(screen.getByLabelText('Project Name'), { target: { value: 'T' } });
    });
    
    // Check that error message is cleared
    await waitFor(() => {
      expect(screen.queryByText('Project Name is required')).not.toBeInTheDocument();
    });
  });

  it('formats large satoshi amounts as BTC', async () => {
    render(<PositionManagement {...defaultProps} />);
    
    // Enter values that result in > 10M sats
    await act(async () => {
      fireEvent.change(screen.getByLabelText('Investment Amount'), { target: { value: '1000' } });
      fireEvent.change(screen.getByLabelText('Price per Token (sats)'), { target: { value: '20000' } });
    });
    
    // Check for BTC formatting
    await waitFor(() => {
      expect(screen.getByText('₿0.20000000')).toBeInTheDocument();
    });
  });
}); 