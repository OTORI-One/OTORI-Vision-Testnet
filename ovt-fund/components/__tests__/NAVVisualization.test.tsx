import { render, screen, fireEvent } from '@testing-library/react';
import NAVVisualization from '../NAVVisualization';

const mockData = [
  {
    name: 'Test Project',
    initial: 1000000,
    current: 2000000,
    change: 100,
    description: 'Test Description'
  }
];

jest.mock('../src/hooks/useBitcoinPrice', () => ({
  useBitcoinPrice: () => ({
    price: 40000,
    loading: false,
    error: null
  })
}));

describe('NAVVisualization', () => {
  const defaultProps = {
    data: mockData,
    totalValue: '₿2.00',
    changePercentage: '+100%',
    baseCurrency: 'btc' as const
  };

  it('renders without crashing', () => {
    render(<NAVVisualization {...defaultProps} />);
    expect(screen.getByText('OTORI Net Asset Value - Tracked by $OVT')).toBeInTheDocument();
  });

  it('displays total value and change percentage', () => {
    render(<NAVVisualization {...defaultProps} />);
    expect(screen.getByText('₿2.00')).toBeInTheDocument();
    expect(screen.getByText('+100%')).toBeInTheDocument();
  });

  it('shows project data in chart', () => {
    render(<NAVVisualization {...defaultProps} />);
    expect(screen.getByText('Test Project')).toBeInTheDocument();
  });

  it('opens TokenExplorerModal on bar click', async () => {
    render(<NAVVisualization {...defaultProps} />);
    const bar = screen.getByRole('button', { name: /Test Project/i });
    fireEvent.click(bar);
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
  });
}); 