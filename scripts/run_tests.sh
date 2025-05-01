#!/bin/bash

# Run tests for social-fi Credit

# Run smart contract tests
echo "Running smart contract tests..."
cd smart-contracts
cargo test
cd ..

# Run backend tests
echo "Running backend tests..."
cd backend
python -m pytest
cd ..

# Run frontend tests
echo "Running frontend tests..."
cd frontend
npm test
cd ..

echo "All tests completed!"
