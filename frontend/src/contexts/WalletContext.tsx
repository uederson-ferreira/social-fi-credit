//frontend/src/contexts/WalletContext.tsx

import React, { createContext, useState, useEffect, useContext, ReactNode } from 'react';
import { ExtensionProvider } from '@multiversx/sdk-extension-provider';
import { WalletConnectV2Provider } from '@multiversx/sdk-wallet-connect-provider';
import { ProxyNetworkProvider } from '@multiversx/sdk-network-providers';
import { Address } from '@multiversx/sdk-core';

// Define the network endpoints (adjust for testnet/mainnet)
const apiUrl = 'https://devnet-api.multiversx.com'; 
const networkProvider = new ProxyNetworkProvider(apiUrl);

type WalletContextType = {
  connected: boolean;
  address: string | null;
  balance: string | null;
  provider: any | null;
  connect: (providerType: 'extension' | 'walletconnect') => Promise<void>;
  disconnect: () => void;
  signTransaction: (transaction: any) => Promise<any>;
};

const defaultContext: WalletContextType = {
  connected: false,
  address: null,
  balance: null,
  provider: null,
  connect: async () => {},
  disconnect: () => {},
  signTransaction: async () => ({}),
};

const WalletContext = createContext<WalletContextType>(defaultContext);

export const useWallet = () => useContext(WalletContext);

type WalletContextProviderProps = {
  children: ReactNode;
};

export const WalletContextProvider: React.FC<WalletContextProviderProps> = ({ children }) => {
  const [connected, setConnected] = useState(false);
  const [address, setAddress] = useState<string | null>(null);
  const [balance, setBalance] = useState<string | null>(null);
  const [provider, setProvider] = useState<any | null>(null);

  // Initialize wallet from local storage on mount
  useEffect(() => {
    const savedAddress = localStorage.getItem('walletAddress');
    const savedProviderType = localStorage.getItem('walletProvider');
    
    if (savedAddress && savedProviderType) {
      connect(savedProviderType as 'extension' | 'walletconnect');
    }
  }, []);

  // Update balance when address changes
  useEffect(() => {
    if (address) {
      fetchBalance();
    }
  }, [address]);

  const fetchBalance = async () => {
    if (!address) return;
    
    try {
      const account = await networkProvider.getAccount(new Address(address));
      setBalance(account.balance.toString());
    } catch (error) {
      console.error('Error fetching balance:', error);
    }
  };

  const connect = async (providerType: 'extension' | 'walletconnect') => {
    try {
      let walletProvider;
      
      if (providerType === 'extension') {
        walletProvider = ExtensionProvider.getInstance();
        await walletProvider.init();
      } else {
        walletProvider = new WalletConnectV2Provider(
          apiUrl,
          'social-fi-credit', // Project name
          1, // Chain ID (1 for MultiversX mainnet)
          { 
            onClientLogin: () => {}, 
            onClientLogout: () => disconnect() 
          }
        );
        await walletProvider.init();
      }
      
      await walletProvider.login();
      const walletAddress = walletProvider.account.address;
      
      setProvider(walletProvider);
      setAddress(walletAddress);
      setConnected(true);
      
      // Save to local storage
      localStorage.setItem('walletAddress', walletAddress);
      localStorage.setItem('walletProvider', providerType);
      
    } catch (error) {
      console.error('Wallet connection error:', error);
    }
  };

  const disconnect = () => {
    if (provider) {
      provider.logout();
    }
    
    setProvider(null);
    setAddress(null);
    setBalance(null);
    setConnected(false);
    
    // Clear local storage
    localStorage.removeItem('walletAddress');
    localStorage.removeItem('walletProvider');
  };

  const signTransaction = async (transaction: any) => {
    if (!provider) {
      throw new Error('Wallet not connected');
    }
    
    return await provider.signTransaction(transaction);
  };

  return (
    <WalletContext.Provider
      value={{
        connected,
        address,
        balance,
        provider,
        connect,
        disconnect,
        signTransaction,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
};
