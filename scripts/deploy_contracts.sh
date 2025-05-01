#!/bin/bash

# Deploy social-fi Credit smart contracts to MultiversX devnet

# Check if mxpy is installed
if ! command -v mxpy &> /dev/null; then
    echo "Error: mxpy is not installed. Please install MultiversX SDK first."
    exit 1
fi

# Check if the wallet is defined
if [ -z "$1" ]; then
    echo "Usage: $0 <wallet.pem>"
    exit 1
fi

WALLET_PEM=$1
CHAIN="devnet"

# Deploy Reputation Score contract
echo "Deploying Reputation Score contract..."
REPUTATION_SCORE=$(mxpy contract deploy \
    --bytecode=./smart-contracts/reputation-score/wasm/reputation-score.wasm \
    --pem=$WALLET_PEM \
    --gas-limit=60000000 \
    --arguments 0 1000 \
    --chain=$CHAIN \
    --send \
    --outfile=reputation-score-deploy.json)

# Extract the contract address
REPUTATION_SCORE_ADDRESS=$(cat reputation-score-deploy.json | jq -r '.emittedTransactionHash')
echo "Reputation Score contract deployed at: $REPUTATION_SCORE_ADDRESS"

# Deploy Loan Controller contract
echo "Deploying Loan Controller contract..."
LOAN_CONTROLLER=$(mxpy contract deploy \
    --bytecode=./smart-contracts/loan-controller/wasm/loan-controller.wasm \
    --pem=$WALLET_PEM \
    --gas-limit=60000000 \
    --arguments $REPUTATION_SCORE_ADDRESS 50 1000 \
    --chain=$CHAIN \
    --send \
    --outfile=loan-controller-deploy.json)

# Extract the contract address
LOAN_CONTROLLER_ADDRESS=$(cat loan-controller-deploy.json | jq -r '.emittedTransactionHash')
echo "Loan Controller contract deployed at: $LOAN_CONTROLLER_ADDRESS"

# Deploy Liquidity Pool contract
echo "Deploying Liquidity Pool contract..."
LIQUIDITY_POOL=$(mxpy contract deploy \
    --bytecode=./smart-contracts/liquidity-pool/wasm/liquidity-pool.wasm \
    --pem=$WALLET_PEM \
    --gas-limit=60000000 \
    --arguments $LOAN_CONTROLLER_ADDRESS \
    --chain=$CHAIN \
    --send \
    --outfile=liquidity-pool-deploy.json)

# Extract the contract address
LIQUIDITY_POOL_ADDRESS=$(cat liquidity-pool-deploy.json | jq -r '.emittedTransactionHash')
echo "Liquidity Pool contract deployed at: $LIQUIDITY_POOL_ADDRESS"

# Deploy Debt Token contract
echo "Deploying Debt Token contract..."
DEBT_TOKEN=$(mxpy contract deploy \
    --bytecode=./smart-contracts/debt-token/wasm/debt-token.wasm \
    --pem=$WALLET_PEM \
    --gas-limit=60000000 \
    --arguments $LOAN_CONTROLLER_ADDRESS \
    --chain=$CHAIN \
    --send \
    --outfile=debt-token-deploy.json)

# Extract the contract address
DEBT_TOKEN_ADDRESS=$(cat debt-token-deploy.json | jq -r '.emittedTransactionHash')
echo "Debt Token contract deployed at: $DEBT_TOKEN_ADDRESS"

# Update .env file with the contract addresses
echo "Updating .env file with contract addresses..."
sed -i "s/REPUTATION_SCORE_ADDRESS=.*/REPUTATION_SCORE_ADDRESS=$REPUTATION_SCORE_ADDRESS/" .env
sed -i "s/LOAN_CONTROLLER_ADDRESS=.*/LOAN_CONTROLLER_ADDRESS=$LOAN_CONTROLLER_ADDRESS/" .env
sed -i "s/LIQUIDITY_POOL_ADDRESS=.*/LIQUIDITY_POOL_ADDRESS=$LIQUIDITY_POOL_ADDRESS/" .env
sed -i "s/DEBT_TOKEN_ADDRESS=.*/DEBT_TOKEN_ADDRESS=$DEBT_TOKEN_ADDRESS/" .env

echo "All contracts deployed successfully!"
echo "Contract addresses have been updated in the .env file"

# Save deployment info to a JSON file
echo "{
  \"reputation_score\": \"$REPUTATION_SCORE_ADDRESS\",
  \"loan_controller\": \"$LOAN_CONTROLLER_ADDRESS\",
  \"liquidity_pool\": \"$LIQUIDITY_POOL_ADDRESS\",
  \"debt_token\": \"$DEBT_TOKEN_ADDRESS\",
  \"chain\": \"$CHAIN\",
  \"deployed_at\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"
}" > deployed-contracts.json

echo "Deployment information saved to deployed-contracts.json"
