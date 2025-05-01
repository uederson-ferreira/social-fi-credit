//frontend/src/components/dashboard/ScoreDisplay.tsx

import React from 'react';
import { useUser } from '../../contexts/UserContext.tsx';

const ScoreDisplay: React.FC = () => {
  const { score, loading, error, refreshScore, twitterConnected } = useUser();

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6 animate-pulse">
        <div className="h-8 bg-gray-200 rounded mb-4 w-3/4"></div>
        <div className="h-28 bg-gray-200 rounded-full w-28 mx-auto mb-4"></div>
        <div className="h-6 bg-gray-200 rounded mb-2 w-1/2 mx-auto"></div>
        <div className="h-10 bg-gray-200 rounded w-full mt-4"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6">
        <div className="text-red-500 mb-4">
          <p>{error}</p>
        </div>
        <button 
          onClick={() => refreshScore()}
          className="bg-blue-500 hover:bg-blue-600 text-white py-2 px-4 rounded-md"
        >
          Try Again
        </button>
      </div>
    );
  }

  if (!score) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6">
        <p className="text-gray-500 mb-4">Connect your wallet to view your Community Score</p>
      </div>
    );
  }

  // Calculate percentage for the circular progress
  const percentage = (score.current / score.max) * 100;
  
  // Determine score category
  let scoreCategory = 'Starter';
  let scoreColor = 'text-yellow-500';
  
  if (percentage >= 75) {
    scoreCategory = 'Excellent';
    scoreColor = 'text-green-500';
  } else if (percentage >= 50) {
    scoreCategory = 'Good';
    scoreColor = 'text-blue-500';
  } else if (percentage >= 25) {
    scoreCategory = 'Average';
    scoreColor = 'text-orange-500';
  }

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h2 className="text-xl font-semibold mb-4">Community Score</h2>
      
      <div className="flex flex-col items-center mb-6">
        {/* Circular score display */}
        <div className="relative h-32 w-32 mb-4">
          <svg className="h-full w-full" viewBox="0 0 100 100">
            {/* Background circle */}
            <circle
              className="text-gray-200"
              strokeWidth="10"
              stroke="currentColor"
              fill="transparent"
              r="40"
              cx="50"
              cy="50"
            />
            {/* Progress circle */}
            <circle
              className={scoreColor.replace('text', 'stroke')}
              strokeWidth="10"
              strokeDasharray={`${2.5 * Math.PI * 40}`}
              strokeDashoffset={`${((100 - percentage) / 100) * 2.5 * Math.PI * 40}`}
              strokeLinecap="round"
              stroke="currentColor"
              fill="transparent"
              r="40"
              cx="50"
              cy="50"
            />
            {/* Score text */}
            <text
              x="50"
              y="50"
              fontFamily="Verdana"
              fontSize="20"
              textAnchor="middle"
              alignmentBaseline="middle"
            >
              {score.current}
            </text>
          </svg>
        </div>
        
        <p className={`text-lg font-medium ${scoreColor}`}>
          {scoreCategory}
        </p>
        
        <p className="text-gray-500 text-sm mt-1">
          {score.current} / {score.max} points
        </p>
      </div>
      
      <div className="border-t pt-4">
        <div className="flex justify-between mb-2">
          <span className="text-gray-600">Loan Eligible:</span>
          <span className={score.eligibleForLoan ? "text-green-500 font-medium" : "text-red-500 font-medium"}>
            {score.eligibleForLoan ? "Yes" : "No"}
          </span>
        </div>
        
        <div className="flex justify-between">
          <span className="text-gray-600">Max Loan Amount:</span>
          <span className="font-medium">{score.maxLoanAmount} EGLD</span>
        </div>
      </div>
      
      {!twitterConnected && (
        <div className="mt-4 p-3 bg-blue-50 text-blue-700 rounded-md">
          <p className="text-sm">Connect your Twitter account to increase your score</p>
        </div>
      )}
      
      <button 
        onClick={() => refreshScore()}
        className="mt-4 w-full bg-gray-100 hover:bg-gray-200 text-gray-800 py-2 px-4 rounded-md text-sm flex items-center justify-center"
      >
        <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
        </svg>
        Refresh Score
      </button>
    </div>
  );
};

export default ScoreDisplay;