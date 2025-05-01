#!/bin/bash

# Set up environment for social-fi Credit development

# Check if .env file exists, create from example if not
if [ ! -f .env ]; then
    echo "Creating .env file from example..."
    cp .env.example .env
    echo "Please update the .env file with your contract addresses and Twitter API credentials"
fi

# Install Python dependencies
echo "Installing Python dependencies..."
cd backend
pip install -r requirements.txt
cd ..

# Install JavaScript dependencies
echo "Installing JavaScript dependencies..."
cd frontend
npm install
cd ..

# Check if Docker is installed
if command -v docker >/dev/null 2>&1; then
    echo "Docker is installed, you can use docker-compose up to start the services"
else
    echo "Docker is not installed. Please install Docker to use docker-compose"
fi

echo "Environment setup complete!"
