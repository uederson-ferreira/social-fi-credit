//frontend/src/pages/Home.tsx
import React from 'react';
import { Link } from 'react-router-dom';
import { useWallet } from '../contexts/WalletContext.tsx';

const Home: React.FC = () => {
  const { connected, connect } = useWallet();

  return (
    <div className="flex flex-col items-center">
      {/* Hero Section */}
      <section className="w-full bg-gradient-to-r from-blue-500 to-purple-600 text-white py-16 px-4 rounded-lg">
        <div className="max-w-4xl mx-auto text-center">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">
            Your Social Reputation Is Now Your Credit Score
          </h1>
          <p className="text-xl md:text-2xl mb-8 opacity-90">
            social-fi Credit revolutionizes DeFi with zero-collateral loans based on your social reputation.
          </p>
          
          {!connected ? (
            <div>
              <button 
                onClick={() => connect('extension')}
                className="bg-white text-blue-600 hover:bg-blue-50 font-medium py-3 px-8 rounded-lg text-lg shadow-lg mr-4"
              >
                Connect Wallet
              </button>
              <Link 
                to="/loans" 
                className="inline-block mt-4 md:mt-0 bg-transparent border-2 border-white hover:bg-white hover:text-blue-600 text-white font-medium py-3 px-8 rounded-lg text-lg transition-colors"
              >
                Learn More
              </Link>
            </div>
          ) : (
            <div>
              <Link 
                to="/dashboard" 
                className="bg-white text-blue-600 hover:bg-blue-50 font-medium py-3 px-8 rounded-lg text-lg shadow-lg mr-4"
              >
                Go to Dashboard
              </Link>
              <Link 
                to="/loans" 
                className="inline-block mt-4 md:mt-0 bg-transparent border-2 border-white hover:bg-white hover:text-blue-600 text-white font-medium py-3 px-8 rounded-lg text-lg transition-colors"
              >
                Get a Loan
              </Link>
            </div>
          )}
        </div>
      </section>

      {/* How It Works Section */}
      <section className="w-full py-16 px-4">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">How It Works</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {/* Step 1 */}
            <div className="bg-white rounded-lg shadow-md p-6 text-center">
              <div className="w-16 h-16 bg-blue-100 text-blue-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">1</div>
              <h3 className="text-xl font-semibold mb-3">Connect Your Social</h3>
              <p className="text-gray-600">Link your Twitter account to start building your reputation through the #ElizaOS hashtag.</p>
            </div>
            
            {/* Step 2 */}
            <div className="bg-white rounded-lg shadow-md p-6 text-center">
              <div className="w-16 h-16 bg-purple-100 text-purple-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">2</div>
              <h3 className="text-xl font-semibold mb-3">Build Reputation</h3>
              <p className="text-gray-600">Engage with the community by sharing resources, answering questions, and contributing positively.</p>
            </div>
            
            {/* Step 3 */}
            <div className="bg-white rounded-lg shadow-md p-6 text-center">
              <div className="w-16 h-16 bg-green-100 text-green-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">3</div>
              <h3 className="text-xl font-semibold mb-3">Access Credit</h3>
              <p className="text-gray-600">Use your Community Score to access loans without traditional collateral requirements.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="w-full py-16 px-4 bg-gray-50">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">Key Features</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path fillRule="evenodd" d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Zero-Collateral Loans</h3>
                <p className="text-gray-600">Access crypto loans without locking up your assets as collateral.</p>
              </div>
            </div>
            
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Community-Based Trust</h3>
                <p className="text-gray-600">Your reputation within the community determines your creditworthiness.</p>
              </div>
            </div>
            
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path fillRule="evenodd" d="M12 7a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0V8.414l-4.293 4.293a1 1 0 01-1.414 0L8 10.414l-4.293 4.293a1 1 0 01-1.414-1.414l5-5a1 1 0 011.414 0L11 10.586 14.586 7H12z" clipRule="evenodd"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Dynamic Scoring</h3>
                <p className="text-gray-600">Real-time updates to your Community Score based on social interactions.</p>
              </div>
            </div>
            
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path d="M3 12v3c0 1.657 3.134 3 7 3s7-1.343 7-3v-3c0 1.657-3.134 3-7 3s-7-1.343-7-3z"></path>
                  <path d="M3 7v3c0 1.657 3.134 3 7 3s7-1.343 7-3V7c0 1.657-3.134 3-7 3S3 8.657 3 7z"></path>
                  <path d="M17 5c0 1.657-3.134 3-7 3S3 6.657 3 5s3.134-3 7-3 7 1.343 7 3z"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Risk-Based Pools</h3>
                <p className="text-gray-600">Different liquidity pools based on risk tolerance with varying interest rates.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="w-full py-16 px-4">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl font-bold mb-6">Ready to Get Started?</h2>
          <p className="text-xl text-gray-600 mb-8">
            Join the revolution in decentralized finance and start building your credit today.
          </p>
          
          {!connected ? (
            <button 
              onClick={() => connect('extension')}
              className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-8 rounded-lg text-lg shadow-lg"
            >
              Connect Wallet
            </button>
          ) : (
            <Link 
              to="/dashboard" 
              className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-8 rounded-lg text-lg shadow-lg"
            >
              View Dashboard
            </Link>
          )}
        </div>
      </section>
    </div>
  );
};

export default Home;
