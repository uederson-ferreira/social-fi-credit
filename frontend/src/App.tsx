import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { WalletContextProvider } from './contexts/WalletContext.tsx';
import { UserContextProvider } from './contexts/UserContext.tsx';
import Header from './components/common/Header.tsx';
import Footer from './components/common/Footer.tsx';
import Home from './pages/Home.tsx';
import Dashboard from './pages/Dashboard.tsx';
import Loans from './pages/Loans.tsx';
import Pools from './pages/Pools.tsx';
import Profile from './pages/Profile.tsx';
import NFTMarketplace from './pages/NFTMarketplace.tsx';
import './assets/styles/global.css';

const App: React.FC = () => {
  return (
    <Router>
      <WalletContextProvider>
        <UserContextProvider>
          <div className="flex flex-col min-h-screen">
            <Header />
            <main className="flex-grow container mx-auto px-4 py-8">
              <Routes>
                <Route path="/" element={<Home />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/loans" element={<Loans />} />
                <Route path="/pools" element={<Pools />} />
                <Route path="/profile" element={<Profile />} />
                <Route path="/marketplace" element={<NFTMarketplace />} />
              </Routes>
            </main>
            <Footer />
          </div>
        </UserContextProvider>
      </WalletContextProvider>
    </Router>
  );
};

export default App;