import axios from 'axios';

// Create axios instance
const api = axios.create({
  baseURL: process.env.REACT_APP_API_URL || 'http://localhost:8000',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Get user profile
export const getUser = async (address: string) => {
  const response = await api.get(`/api/users/${address}`);
  return response.data;
};

// Get user score
export const getUserScore = async (address: string) => {
  const response = await api.get(`/api/users/${address}/score`);
  return response.data;
};

// Connect Twitter account
export const connectTwitter = async (address: string, twitterHandle: string, oauthToken: string) => {
  const response = await api.post(`/api/users/${address}/connect-twitter`, { 
    twitterHandle, 
    oauthToken 
  });
  return response.data;
};

// Get Twitter stats
export const getTwitterStats = async (address: string) => {
  const response = await api.get(`/api/users/${address}/twitter-stats`);
  return response.data;
};

// Get loans
export const getLoans = async (address?: string, status?: string) => {
  let url = '/api/loans';
  
  // Add query params if provided
  const params = new URLSearchParams();
  if (address) params.append('address', address);
  if (status) params.append('status', status);
  
  if (params.toString()) {
    url += `?${params.toString()}`;
  }
  
  const response = await api.get(url);
  return response.data;
};

// Get loan by ID
export const getLoan = async (loanId: string) => {
  const response = await api.get(`/api/loans/${loanId}`);
  return response.data;
};

// Request a loan
export const requestLoan = async (amount: string, durationDays: number, tokenId: string) => {
  const response = await api.post('/api/loans/request', {
    amount,
    duration_days: durationDays,
    token_id: tokenId,
  });
  return response.data;
};

// Repay a loan
export const repayLoan = async (loanId: string, tokenId: string) => {
  const response = await api.post('/api/loans/repay', {
    loan_id: loanId,
    token_id: tokenId,
  });
  return response.data;
};

// Calculate interest
export const calculateInterest = async (amount: string, address: string) => {
  const response = await api.get('/api/loans/calculate-interest', {
    params: { amount, address }
  });
  return response.data;
};

// Get pools
export const getPools = async () => {
  const response = await api.get('/api/pools');
  return response.data;
};

// Get pool by ID
export const getPool = async (poolId: string) => {
  const response = await api.get(`/api/pools/${poolId}`);
  return response.data;
};

// Provide liquidity
export const provideLiquidity = async (poolId: string, amount: string, tokenId: string) => {
  const response = await api.post('/api/pools/provide', {
    pool_id: poolId,
    amount,
    token_id: tokenId,
  });
  return response.data;
};

// Withdraw liquidity
export const withdrawLiquidity = async (poolId: string, amount: string) => {
  const response = await api.post('/api/pools/withdraw', {
    pool_id: poolId,
    amount,
  });
  return response.data;
};
