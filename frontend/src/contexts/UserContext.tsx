//frontend/src/contexts/UserContext.tsx

import React, { createContext, useState, useEffect, useContext, ReactNode } from 'react';
import { useWallet } from './WalletContext.tsx';
import { getUser, getUserScore } from '../services/api.ts';

type UserScore = {
  current: number;
  max: number;
  eligibleForLoan: boolean;
  maxLoanAmount: string;
};

type UserContextType = {
  score: UserScore | null;
  loading: boolean;
  error: string | null;
  refreshScore: () => Promise<void>;
  twitterConnected: boolean;
  connectTwitter: () => Promise<void>;
};

const defaultContext: UserContextType = {
  score: null,
  loading: false,
  error: null,
  refreshScore: async () => {},
  twitterConnected: false,
  connectTwitter: async () => {},
};

const UserContext = createContext<UserContextType>(defaultContext);

export const useUser = () => useContext(UserContext);

type UserContextProviderProps = {
  children: ReactNode;
};

export const UserContextProvider: React.FC<UserContextProviderProps> = ({ children }) => {
  const { address, connected } = useWallet();
  const [score, setScore] = useState<UserScore | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [twitterConnected, setTwitterConnected] = useState(false);

  // Load user data when wallet connects
  useEffect(() => {
    if (connected && address) {
      refreshScore();
      checkTwitterConnection();
    } else {
      setScore(null);
      setTwitterConnected(false);
    }
  }, [connected, address]);

  const refreshScore = async () => {
    if (!address) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const userScore = await getUserScore(address);
      setScore(userScore);
    } catch (err) {
      console.error('Error fetching user score:', err);
      setError('Failed to load your reputation score. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const checkTwitterConnection = async () => {
    if (!address) return;
    
    try {
      const userData = await getUser(address);
      setTwitterConnected(!!userData.twitterId);
    } catch (err) {
      console.error('Error checking Twitter connection:', err);
      setTwitterConnected(false);
    }
  };

  const connectTwitter = async () => {
    // This would typically open Twitter OAuth flow
    // For now, just mock it
    setTwitterConnected(true);
    return Promise.resolve();
  };

  return (
    <UserContext.Provider
      value={{
        score,
        loading,
        error,
        refreshScore,
        twitterConnected,
        connectTwitter,
      }}
    >
      {children}
    </UserContext.Provider>
  );
};