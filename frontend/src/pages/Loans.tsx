import React, { useState, useEffect } from 'react';
import { useWallet } from '../contexts/WalletContext.tsx';
import { useUser } from '../contexts/UserContext.tsx';
import { getLoans, requestLoan, calculateInterest } from '../services/api.ts';

interface LoanFormData {
  amount: string;
  durationDays: number;
  tokenId: string;
}

interface LoanCalculation {
  interestRate: number;
  repaymentAmount: string;
  dueDate: string;
}

const Loans: React.FC = () => {
  const { connected, address } = useWallet();
  const { score } = useUser();
  
  const [activeLoans, setActiveLoans] = useState([]);
  const [loanHistory, setLoanHistory] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const [loanForm, setLoanForm] = useState<LoanFormData>({
    amount: '',
    durationDays: 30,
    tokenId: 'EGLD',
  });
  
  const [loanCalculation, setLoanCalculation] = useState<LoanCalculation | null>(null);
  const [calculationLoading, setCalculationLoading] = useState(false);
  
  // Load loans when address changes
  useEffect(() => {
    if (connected && address) {
      fetchLoans();
    }
  }, [connected, address]);
  
  // Calculate loan details when form changes
  useEffect(() => {
    if (connected && address && loanForm.amount && parseFloat(loanForm.amount) > 0) {
      calculateLoanDetails();
    } else {
      setLoanCalculation(null);
    }
  }, [loanForm, address]);
  
  const fetchLoans = async () => {
    if (!address) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // Fetch active loans
      const active = await getLoans(address, 'Active');
      setActiveLoans(active);
      
      // Fetch loan history (repaid and defaulted)
      const history = await getLoans(address);
      setLoanHistory(history.filter((loan: any) => loan.status !== 'Active'));
    } catch (err) {
      console.error('Error fetching loans:', err);
      setError('Failed to load your loans. Please try again.');
    } finally {
      setLoading(false);
    }
  };
  
  const calculateLoanDetails = async () => {
    if (!address || !loanForm.amount || parseFloat(loanForm.amount) <= 0) return;
    
    setCalculationLoading(true);
    
    try {
      const calculation = await calculateInterest(loanForm.amount, address);
      
      // Calculate due date
      const dueDate = new Date();
      dueDate.setDate(dueDate.getDate() + loanForm.durationDays);
      
      setLoanCalculation({
        ...calculation,
        dueDate: dueDate.toLocaleDateString(),
      });
    } catch (err) {
      console.error('Error calculating loan details:', err);
      setLoanCalculation(null);
    } finally {
      setCalculationLoading(false);
    }
  };
  
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    
    setLoanForm({
      ...loanForm,
      [name]: name === 'durationDays' ? parseInt(value) : value,
    });
  };
  
  const handleLoanRequest = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!connected || !address) {
      setError('Please connect your wallet first');
      return;
    }
    
    if (!score || !score.eligibleForLoan) {
      setError('Your score is not high enough to get a loan');
      return;
    }
    
    const amount = parseFloat(loanForm.amount);
    if (isNaN(amount) || amount <= 0) {
      setError('Please enter a valid amount');
      return;
    }
    
    const maxAmount = parseFloat(score.maxLoanAmount);
    if (amount > maxAmount) {
      setError(`Amount exceeds your maximum loan amount of ${maxAmount} EGLD`);
      return;
    }
    
    setLoading(true);
    setError(null);
    
    try {
      const response = await requestLoan(
        loanForm.amount,
        loanForm.durationDays,
        loanForm.tokenId
      );
      
      // In a real implementation, this would trigger a transaction signing
      // and then refresh the loans list
      
      alert('Loan request successful! Please sign the transaction in your wallet.');
      
      // Reset form
      setLoanForm({
        amount: '',
        durationDays: 30,
        tokenId: 'EGLD',
      });
      
      // Refresh loans after a short delay
      setTimeout(fetchLoans, 2000);
    } catch (err) {
      console.error('Error requesting loan:', err);
      setError('Failed to process your loan request. Please try again.');
    } finally {
      setLoading(false);
    }
  };
  
  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center py-12">
        <div className="text-center">
          <h1 className="text-3xl font-bold mb-4">Connect Wallet to Access Loans</h1>
          <p className="text-gray-600 mb-8">
            Please connect your MultiversX wallet to view available loans and apply for credit.
          </p>
        </div>
      </div>
    );
  }
  
  return (
    <div>
      <h1 className="text-3xl font-bold mb-6">Loans</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Loan Application Form */}
        <div className="col-span-1 md:col-span-2 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">Apply for a Loan</h2>
          
          {error && (
            <div className="bg-red-100 text-red-700 p-3 rounded-md mb-4">
              {error}
            </div>
          )}
          
          {!score?.eligibleForLoan && (
            <div className="bg-yellow-100 text-yellow-700 p-3 rounded-md mb-4">
              Your community score is not yet high enough to qualify for a loan. 
              Continue engaging with the community to increase your score.
            </div>
          )}
          
          <form onSubmit={handleLoanRequest}>
            <div className="mb-4">
              <label className="block text-gray-700 text-sm font-medium mb-2">
                Loan Amount (EGLD)
              </label>
              <input
                type="number"
                name="amount"
                value={loanForm.amount}
                onChange={handleInputChange}
                placeholder="Enter amount"
                className="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                step="0.01"
                min="0.1"
                max={score?.maxLoanAmount || 0}
                disabled={!score?.eligibleForLoan || loading}
                required
              />
              {score && (
                <p className="mt-1 text-sm text-gray-500">
                  Maximum amount: {score.maxLoanAmount} EGLD
                </p>
              )}
            </div>
            
            <div className="mb-4">
              <label className="block text-gray-700 text-sm font-medium mb-2">
                Loan Duration
              </label>
              <select
                name="durationDays"
                value={loanForm.durationDays}
                onChange={handleInputChange}
                className="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                disabled={!score?.eligibleForLoan || loading}
              >
                <option value={7}>7 days</option>
                <option value={14}>14 days</option>
                <option value={30}>30 days</option>
                <option value={60}>60 days</option>
                <option value={90}>90 days</option>
              </select>
            </div>
            
            <div className="mb-6">
              <label className="block text-gray-700 text-sm font-medium mb-2">
                Token
              </label>
              <select
                name="tokenId"
                value={loanForm.tokenId}
                onChange={handleInputChange}
                className="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                disabled={!score?.eligibleForLoan || loading}
              >
                <option value="EGLD">EGLD (MultiversX)</option>
                {/* More token options would be here in a real implementation */}
              </select>
            </div>
            
            {loanCalculation && (
              <div className="mb-6 bg-gray-50 p-4 rounded-md">
                <h3 className="text-lg font-medium mb-2">Loan Details</h3>
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div className="text-gray-600">Interest Rate:</div>
                  <div className="font-medium">{loanCalculation.interestRate}%</div>
                  
                  <div className="text-gray-600">Repayment Amount:</div>
                  <div className="font-medium">{loanCalculation.repaymentAmount} EGLD</div>
                  
                  <div className="text-gray-600">Due Date:</div>
                  <div className="font-medium">{loanCalculation.dueDate}</div>
                </div>
              </div>
            )}
            
            <button
              type="submit"
              className={`w-full py-3 rounded-md font-medium ${
                score?.eligibleForLoan
                  ? 'bg-blue-600 hover:bg-blue-700 text-white'
                  : 'bg-gray-300 text-gray-500 cursor-not-allowed'
              }`}
              disabled={!score?.eligibleForLoan || loading || calculationLoading}
            >
              {loading ? 'Processing...' : 'Apply for Loan'}
            </button>
          </form>
        </div>
        
        {/* Loan Info */}
        <div className="col-span-1 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">How It Works</h2>
          
          <div className="space-y-4">
            <div>
              <h3 className="font-medium text-blue-600">Zero Collateral</h3>
              <p className="text-sm text-gray-600">
                Our loans don't require traditional collateral. Your community reputation is your credit score.
              </p>
            </div>
            
            <div>
              <h3 className="font-medium text-blue-600">Dynamic Interest Rates</h3>
              <p className="text-sm text-gray-600">
                Interest rates are determined by your community score. Higher scores mean lower rates.
              </p>
            </div>
            
            <div>
              <h3 className="font-medium text-blue-600">Flexible Repayment</h3>
              <p className="text-sm text-gray-600">
                Choose loan duration from 7 to 90 days. Repay anytime before the due date.
              </p>
            </div>
            
            <div>
              <h3 className="font-medium text-blue-600">Reputation Building</h3>
              <p className="text-sm text-gray-600">
                Timely repayments boost your community score, increasing future loan limits.
              </p>
            </div>
          </div>
        </div>
      </div>
      
      {/* Active Loans */}
      <div className="mt-8 bg-white rounded-lg shadow-md p-6">
        <h2 className="text-xl font-semibold mb-4">Your Active Loans</h2>
        
        {loading ? (
          <div className="text-center py-4">
            <div className="spinner mx-auto"></div>
            <p className="text-gray-500 mt-2">Loading your loans...</p>
          </div>
        ) : activeLoans.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Amount
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Interest Rate
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Due Date
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Action
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {/* Loan items would be rendered here in a real implementation */}
                <tr>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">1.5 EGLD</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">5.2%</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">May 30, 2025</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                      Active
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                    <button className="text-blue-600 hover:text-blue-900">
                      Repay
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="text-center py-6">
            <p className="text-gray-500">You don't have any active loans</p>
          </div>
        )}
      </div>
      
      {/* Loan History */}
      <div className="mt-8 bg-white rounded-lg shadow-md p-6">
        <h2 className="text-xl font-semibold mb-4">Loan History</h2>
        
        {loading ? (
          <div className="text-center py-4">
            <div className="spinner mx-auto"></div>
            <p className="text-gray-500 mt-2">Loading your loan history...</p>
          </div>
        ) : loanHistory.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Amount
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Interest Rate
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Due Date
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {/* Loan history items would be rendered here in a real implementation */}
                <tr>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">0.5 EGLD</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">7.5%</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">April 15, 2025</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800">
                      Repaid
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="text-center py-6">
            <p className="text-gray-500">You don't have any loan history</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default Loans;
