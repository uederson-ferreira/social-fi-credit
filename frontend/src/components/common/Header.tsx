import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useWallet } from '../../contexts/WalletContext.tsx';
import { useUser } from '../../contexts/UserContext.tsx';

const Header: React.FC = () => {
  const { connected, address, balance, connect, disconnect } = useWallet();
  const { score } = useUser();
  const location = useLocation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [walletMenuOpen, setWalletMenuOpen] = useState(false);

  // Format wallet address for display
  const formatAddress = (addr: string): string => {
    if (!addr) return '';
    return `${addr.substring(0, 6)}...${addr.substring(addr.length - 4)}`;
  };
  
  // Format balance for display
  const formatBalance = (bal: string): string => {
    if (!bal) return '0';
    // Convert wei to EGLD (1 EGLD = 10^18 wei)
    const egld = parseInt(bal) / 1000000000000000000;
    return egld.toFixed(4);
  };

  // Navigation links
  const navLinks = [
    { name: 'Home', path: '/' },
    { name: 'Dashboard', path: '/dashboard' },
    { name: 'Loans', path: '/loans' },
    { name: 'Pools', path: '/pools' },
    { name: 'Marketplace', path: '/marketplace' },
  ];

  // Check if a link is active
  const isActive = (path: string): boolean => {
    if (path === '/' && location.pathname === '/') return true;
    return path !== '/' && location.pathname.startsWith(path);
  };

  return (
    <header className="bg-white shadow-sm">
      <div className="container mx-auto px-4 py-3">
        <div className="flex justify-between items-center">
          {/* Logo */}
          <Link to="/" className="flex items-center">
            <span className="text-xl font-bold text-blue-600">social-fi</span>
            <span className="text-xl font-bold text-purple-600">Credit</span>
          </Link>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex space-x-6">
            {navLinks.map((link) => (
              <Link
                key={link.path}
                to={link.path}
                className={`font-medium ${
                  isActive(link.path)
                    ? 'text-blue-600 border-b-2 border-blue-600'
                    : 'text-gray-600 hover:text-blue-600'
                }`}
              >
                {link.name}
              </Link>
            ))}
          </nav>

          {/* Wallet Section */}
          <div className="hidden md:flex items-center space-x-4">
            {connected ? (
              <div className="relative">
                <button
                  onClick={() => setWalletMenuOpen(!walletMenuOpen)}
                  className="flex items-center bg-gray-100 hover:bg-gray-200 rounded-lg px-3 py-2 text-sm font-medium text-gray-800"
                >
                  {score && (
                    <span className="mr-2 flex items-center">
                      <span className="inline-flex items-center justify-center bg-blue-100 text-blue-800 text-xs font-medium rounded-full h-5 w-5 mr-1">
                        {score.current}
                      </span>
                    </span>
                  )}
                  <span>{formatAddress(address || '')}</span>
                  <span className="ml-2 text-green-500 font-semibold">{formatBalance(balance || '0')} EGLD</span>
                  <svg className="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                  </svg>
                </button>

                {walletMenuOpen && (
                  <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-10">
                    <Link
                      to="/profile"
                      className="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                      onClick={() => setWalletMenuOpen(false)}
                    >
                      My Profile
                    </Link>
                    <button
                      onClick={() => {
                        disconnect();
                        setWalletMenuOpen(false);
                      }}
                      className="block w-full text-left px-4 py-2 text-sm text-red-700 hover:bg-gray-100"
                    >
                      Disconnect
                    </button>
                  </div>
                )}
              </div>
            ) : (
              <div className="flex space-x-2">
                <button
                  onClick={() => connect('extension')}
                  className="bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium"
                >
                  Connect Wallet
                </button>
              </div>
            )}
          </div>

          {/* Mobile menu button */}
          <div className="md:hidden flex items-center">
            {connected && (
              <button
                onClick={() => setWalletMenuOpen(!walletMenuOpen)}
                className="flex items-center bg-gray-100 hover:bg-gray-200 rounded-lg px-3 py-2 text-sm font-medium text-gray-800 mr-2"
              >
                <span>{formatAddress(address || '')}</span>
              </button>
            )}
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="text-gray-500 hover:text-gray-600"
            >
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                {mobileMenuOpen ? (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                ) : (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                )}
              </svg>
            </button>
          </div>
        </div>

        {/* Mobile menu */}
        {mobileMenuOpen && (
          <div className="mt-4 md:hidden">
            <div className="flex flex-col space-y-2">
              {navLinks.map((link) => (
                <Link
                  key={link.path}
                  to={link.path}
                  className={`px-3 py-2 rounded-md text-base font-medium ${
                    isActive(link.path)
                      ? 'bg-blue-100 text-blue-700'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-blue-600'
                  }`}
                  onClick={() => setMobileMenuOpen(false)}
                >
                  {link.name}
                </Link>
              ))}
              {!connected && (
                <button
                  onClick={() => {
                    connect('extension');
                    setMobileMenuOpen(false);
                  }}
                  className="mt-2 w-full bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium"
                >
                  Connect Wallet
                </button>
              )}
            </div>
          </div>
        )}
        
        {/* Mobile wallet menu */}
        {walletMenuOpen && mobileMenuOpen && (
          <div className="mt-2 border-t pt-2">
            <div className="flex justify-between items-center mb-2">
              <span className="text-sm text-gray-500">Balance:</span>
              <span className="text-green-500 font-semibold">{formatBalance(balance || '0')} EGLD</span>
            </div>
            {score && (
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-gray-500">Score:</span>
                <span className="flex items-center">
                  <span className="inline-flex items-center justify-center bg-blue-100 text-blue-800 text-sm font-medium rounded-full h-6 w-6">
                    {score.current}
                  </span>
                </span>
              </div>
            )}
            <div className="flex flex-col space-y-2 mt-2">
              <Link
                to="/profile"
                className="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:bg-gray-50 hover:text-blue-600"
                onClick={() => {
                  setWalletMenuOpen(false);
                  setMobileMenuOpen(false);
                }}
              >
                My Profile
              </Link>
              <button
                onClick={() => {
                  disconnect();
                  setWalletMenuOpen(false);
                  setMobileMenuOpen(false);
                }}
                className="block text-left px-3 py-2 rounded-md text-base font-medium text-red-700 hover:bg-gray-50"
              >
                Disconnect
              </button>
            </div>
          </div>
        )}
      </div>
    </header>
  );
};

export default Header;
