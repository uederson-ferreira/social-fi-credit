import React from 'react';
import { useWallet } from '../contexts/WalletContext.tsx';
import ScoreDisplay from '../components/dashboard/ScoreDisplay.tsx';
import { Link } from 'react-router-dom';

const Dashboard: React.FC = () => {
  const { connected, address } = useWallet();

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center py-12">
        <div className="text-center">
          <h1 className="text-3xl font-bold mb-4">Connect Wallet to Access Dashboard</h1>
          <p className="text-gray-600 mb-8">
            Please connect your MultiversX wallet to view your dashboard and access all features.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div>
      <h1 className="text-3xl font-bold mb-6">Dashboard</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Community Score */}
        <div className="col-span-1">
          <ScoreDisplay />
        </div>
        
        {/* Active Loans */}
        <div className="col-span-1 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">Your Active Loans</h2>
          
          <div className="space-y-4">
            {/* Placeholder when no loans */}
            <div className="text-center py-6">
              <p className="text-gray-500 mb-4">You don't have any active loans</p>
              <Link 
                to="/loans" 
                className="inline-block bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium"
              >
                Get a Loan
              </Link>
            </div>
            
            {/* Loan items would be rendered here in a real implementation */}
          </div>
        </div>
        
        {/* Recent Activity */}
        <div className="col-span-1 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">Recent Activity</h2>
          
          <div className="space-y-4">
            {/* Activity items */}
            <div className="border-l-4 border-green-500 pl-4 py-1">
              <p className="text-sm font-medium">Score Increased</p>
              <p className="text-xs text-gray-500">Your community score increased by 5 points</p>
              <p className="text-xs text-gray-400">2 hours ago</p>
            </div>
            
            <div className="border-l-4 border-blue-500 pl-4 py-1">
              <p className="text-sm font-medium">Twitter Connected</p>
              <p className="text-xs text-gray-500">Successfully connected your Twitter account</p>
              <p className="text-xs text-gray-400">1 day ago</p>
            </div>
            
            <div className="border-l-4 border-purple-500 pl-4 py-1">
              <p className="text-sm font-medium">Wallet Connected</p>
              <p className="text-xs text-gray-500">Successfully connected your MultiversX wallet</p>
              <p className="text-xs text-gray-400">1 day ago</p>
            </div>
          </div>
        </div>
      </div>
      
      {/* Twitter Feed */}
      <div className="mt-8 bg-white rounded-lg shadow-md p-6">
        <h2 className="text-xl font-semibold mb-4">Community Activity with #ElizaOS</h2>
        
        <div className="space-y-4">
          {/* Placeholder for Twitter feed */}
          <p className="text-gray-500 text-center py-8">
            Connect your Twitter account to view community activity
          </p>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
