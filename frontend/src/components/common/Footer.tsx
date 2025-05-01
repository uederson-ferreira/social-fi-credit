import React from 'react';
import { Link } from 'react-router-dom';

const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="bg-gray-800 text-white py-8">
      <div className="container mx-auto px-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Logo and Description */}
          <div className="col-span-1 md:col-span-1">
            <Link to="/" className="flex items-center">
              <span className="text-xl font-bold text-blue-400">social-fi</span>
              <span className="text-xl font-bold text-purple-400">Credit</span>
            </Link>
            <p className="mt-3 text-gray-400 text-sm">
              Revolutionizing DeFi with zero-collateral loans based on social reputation.
            </p>
          </div>

          {/* Quick Links */}
          <div className="col-span-1">
            <h3 className="text-lg font-semibold mb-4">Quick Links</h3>
            <ul className="space-y-2">
              <li>
                <Link to="/" className="text-gray-400 hover:text-white transition">
                  Home
                </Link>
              </li>
              <li>
                <Link to="/dashboard" className="text-gray-400 hover:text-white transition">
                  Dashboard
                </Link>
              </li>
              <li>
                <Link to="/loans" className="text-gray-400 hover:text-white transition">
                  Loans
                </Link>
              </li>
              <li>
                <Link to="/pools" className="text-gray-400 hover:text-white transition">
                  Pools
                </Link>
              </li>
              <li>
                <Link to="/marketplace" className="text-gray-400 hover:text-white transition">
                  Marketplace
                </Link>
              </li>
            </ul>
          </div>

          {/* Resources */}
          <div className="col-span-1">
            <h3 className="text-lg font-semibold mb-4">Resources</h3>
            <ul className="space-y-2">
              <li>
                <a 
                  href="https://docs.social-ficredit.io" 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  className="text-gray-400 hover:text-white transition"
                >
                  Documentation
                </a>
              </li>
              <li>
                <a 
                  href="https://github.com/social-fi-credit" 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  className="text-gray-400 hover:text-white transition"
                >
                  GitHub
                </a>
              </li>
              <li>
                <Link to="/faq" className="text-gray-400 hover:text-white transition">
                  FAQ
                </Link>
              </li>
              <li>
                <a 
                  href="https://medium.com/social-fi-credit" 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  className="text-gray-400 hover:text-white transition"
                >
                  Blog
                </a>
              </li>
            </ul>
          </div>

          {/* Community */}
          <div className="col-span-1">
            <h3 className="text-lg font-semibold mb-4">Community</h3>
            <ul className="space-y-2">
              <li>
                <a 
                  href="https://twitter.com/social-fiCredit" 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  className="text-gray-400 hover:text-white transition"
                >
                  Twitter
                </a>
              </li>
              <li>
                <a 
                  href="https://discord.gg/social-ficredit" 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  className="text-gray-400 hover:text-white transition"
                >
                  Discord
                </a>
              </li>
              <li>
                <a 
                  href="https://t.me/social-ficredit" 
                  target="_blank" 
                  rel="noopener noreferrer" 
                  className="text-gray-400 hover:text-white transition"
                >
                  Telegram
                </a>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Section */}
        <div className="border-t border-gray-700 mt-8 pt-6 flex flex-col md:flex-row justify-between items-center">
          <p className="text-gray-400 text-sm">
            &copy; {currentYear} social-fi Credit. All rights reserved.
          </p>
          <div className="flex space-x-4 mt-4 md:mt-0">
            <Link to="/privacy" className="text-gray-400 hover:text-white transition text-sm">
              Privacy Policy
            </Link>
            <Link to="/terms" className="text-gray-400 hover:text-white transition text-sm">
              Terms of Service
            </Link>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
