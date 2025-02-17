// ovt-fund/components/__tests__/TokenExplorerModal.test.tsx

import { render, screen, fireEvent } from '@testing-library/react'
import TokenExplorerModal from '../TokenExplorerModal'
import { generateMockPortfolio } from '../../test-utils/test-utils'

// Mock useBitcoinPrice hook
jest.mock('../../src/hooks/useBitcoinPrice', () => ({
  useBitcoinPrice: () => ({
    price: 50000, // Mock BTC price at $50,000
    loading: false,
    error: null
  })
}));

describe('TokenExplorerModal', () => {
  const mockTokenData = {
    name: 'Test Project',
    description: 'Test Description',
    initial: 1000000,
    current: 2000000,
    change: 100,
    address: '0x1234...5678',
    holdings: '1000 OTORI',
    transactions: [
      {
        date: '2024-03-15',
        type: 'buy' as const,
        amount: '500 OTORI',
        price: '₿0.5'
      }
    ]
  }

  const defaultProps = {
    isOpen: true,
    onClose: jest.fn(),
    tokenData: mockTokenData
  }

  it('renders correctly when open', () => {
    render(<TokenExplorerModal {...defaultProps} />)
    
    // Verify project name as title
    expect(screen.getByText('Test Project')).toBeInTheDocument()
    
    // Verify token data sections
    expect(screen.getByText('Total Holdings')).toBeInTheDocument()
    expect(screen.getByText('1000 OTORI')).toBeInTheDocument()
    expect(screen.getByText('Profit/Loss')).toBeInTheDocument()
    expect(screen.getByText('+100.00%')).toBeInTheDocument()
  })

  it('calls onClose when close button is clicked', () => {
    render(<TokenExplorerModal {...defaultProps} />)
    
    const closeButton = screen.getByTestId('close-icon')
    fireEvent.click(closeButton)
    
    expect(defaultProps.onClose).toHaveBeenCalledTimes(1)
  })

  it('does not render when isOpen is false', () => {
    render(<TokenExplorerModal {...defaultProps} isOpen={false} />)
    
    expect(screen.queryByText('Token Explorer')).not.toBeInTheDocument()
  })

  it('displays loading state when tokenData is loading', () => {
    const loadingProps = {
      ...defaultProps,
      tokenData: {
        ...mockTokenData,
        initial: 0,
        current: 0,
        change: 0,
        holdings: 'Loading...',
        transactions: []
      }
    }
    
    render(<TokenExplorerModal {...loadingProps} />)
    expect(screen.getByText('Loading...')).toBeInTheDocument()
  })

  it('handles error state when tokenData is invalid', () => {
    const errorProps = {
      ...defaultProps,
      tokenData: {
        ...mockTokenData,
        current: 0,
        change: 0,
        holdings: 'Error loading data',
        transactions: []
      }
    }
    
    render(<TokenExplorerModal {...errorProps} />)
    expect(screen.getByText('Error loading data')).toBeInTheDocument()
  })

  it('formats large numbers correctly', () => {
    const largeNumberData = {
      ...mockTokenData,
      initial: 1000000000,
      current: 2500000000,
      holdings: '1,000,000 OTORI'
    }
    
    render(
      <TokenExplorerModal
        {...defaultProps}
        tokenData={largeNumberData}
      />
    )
    
    // Check formatted profit/loss display
    expect(screen.getByText('+150.00%')).toBeInTheDocument()
    expect(screen.getByText('$1,500,000,000.00')).toBeInTheDocument()
    expect(screen.getByText('1,000,000 OTORI')).toBeInTheDocument()
  })

  it('displays project description correctly', () => {
    render(<TokenExplorerModal {...defaultProps} />)
    
    expect(screen.getByText('Test Description')).toBeInTheDocument()
  })

  describe('Currency Toggle Functionality', () => {
    it('displays values in USD by default', () => {
      render(<TokenExplorerModal {...defaultProps} />)
      
      // Check profit/loss display in USD
      expect(screen.getByText('$1,000,000.00')).toBeInTheDocument()
    })

    it('switches to BTC display when baseCurrency is btc', () => {
      render(
        <TokenExplorerModal
          {...defaultProps}
          baseCurrency="btc"
        />
      )
      
      // With BTC price at $50,000, $1,000,000 = 20 BTC
      expect(screen.getByText('₿20.00')).toBeInTheDocument()
    })

    it('handles Bitcoin price loading state', () => {
      jest.spyOn(require('../../src/hooks/useBitcoinPrice'), 'useBitcoinPrice')
        .mockReturnValue({
          price: null,
          loading: true,
          error: null
        })

      render(
        <TokenExplorerModal
          {...defaultProps}
          baseCurrency="btc"
        />
      )
      
      // When BTC price is loading, it uses fallback price of 40,000
      expect(screen.getByText('₿25.00')).toBeInTheDocument()
    })

    it('handles Bitcoin price error state', () => {
      jest.spyOn(require('../../src/hooks/useBitcoinPrice'), 'useBitcoinPrice')
        .mockReturnValue({
          price: null,
          loading: false,
          error: 'Failed to fetch Bitcoin price'
        })

      render(
        <TokenExplorerModal
          {...defaultProps}
          baseCurrency="btc"
        />
      )
      
      // When there's an error, it uses fallback price of 40,000
      expect(screen.getByText('₿25.00')).toBeInTheDocument()
    })

    it('formats transaction history with correct currency', () => {
      render(
        <TokenExplorerModal
          {...defaultProps}
          baseCurrency="btc"
        />
      )
      
      // Transaction price should remain as is since it's already in BTC
      expect(screen.getByText('₿0.5')).toBeInTheDocument()
    })
  })
})