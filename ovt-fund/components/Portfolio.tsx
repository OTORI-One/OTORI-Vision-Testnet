interface PortfolioItem {
  name: string;
  change: number;
  description: string;
}

interface PortfolioProps {
  items: PortfolioItem[];
  initialInvestment?: string;
  isWalletConnected: boolean;
}

export default function Portfolio({ items, initialInvestment, isWalletConnected }: PortfolioProps) {
  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4">Portfolio</h2>
      
      <div className="space-y-4">
        {items.map((item, index) => (
          <div key={index} className="flex items-start justify-between">
            <div>
              <div className="flex items-center space-x-2">
                <span className="text-lg font-medium">{item.name}</span>
                <span className="text-sm text-gray-500">(public)</span>
              </div>
              <p className="text-sm text-gray-600">{item.description}</p>
            </div>
            <div className="text-right">
              <span className="text-green-600">+{item.change}%</span>
            </div>
          </div>
        ))}
      </div>

      {isWalletConnected && initialInvestment && (
        <div className="mt-6 pt-4 border-t">
          <div className="flex justify-between items-center">
            <span className="text-gray-600">Initial Investment</span>
            <span className="font-medium">{initialInvestment}</span>
          </div>
        </div>
      )}
      
      <button className="mt-4 text-blue-600 text-sm hover:text-blue-800">
        Show more
      </button>
    </div>
  );
} 