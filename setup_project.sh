#!/bin/bash

# Script para criar toda a estrutura de pastas e arquivos do projeto social-fi Credit

echo "Criando estrutura de pastas e arquivos para o projeto social-fi Credit..."

# Função para criar diretórios se não existirem
create_dir() {
  if [ ! -d "$1" ]; then
    mkdir -p "$1"
    echo "Criado diretório: $1"
  fi
}

# Função para criar arquivo com conteúdo
create_file() {
  local file_path="$1"
  local content="$2"
  
  # Cria diretório pai se não existir
  local dir_path=$(dirname "$file_path")
  create_dir "$dir_path"
  
  # Cria o arquivo
  echo "$content" > "$file_path"
  echo "Criado arquivo: $file_path"
}

# Raiz do projeto
BASE_DIR="social-fi-credit"
create_dir "$BASE_DIR"

# Copiar README.md já criado
if [ -f "README.md" ]; then
  cp README.md "$BASE_DIR/"
  echo "Copiado README.md para $BASE_DIR/"
fi

# Copiar .gitignore já criado
if [ -f ".gitignore" ]; then
  cp .gitignore "$BASE_DIR/"
  echo "Copiado .gitignore para $BASE_DIR/"
fi

# Criar estrutura principal
create_dir "$BASE_DIR/smart-contracts"
create_dir "$BASE_DIR/smart-contracts/reputation-score/src"
create_dir "$BASE_DIR/smart-contracts/reputation-score/wasm"
create_dir "$BASE_DIR/smart-contracts/loan-controller/src"
create_dir "$BASE_DIR/smart-contracts/loan-controller/wasm"
create_dir "$BASE_DIR/smart-contracts/liquidity-pool/src"
create_dir "$BASE_DIR/smart-contracts/liquidity-pool/wasm"
create_dir "$BASE_DIR/smart-contracts/debt-token/src"
create_dir "$BASE_DIR/smart-contracts/debt-token/wasm"
create_dir "$BASE_DIR/smart-contracts/tests"
create_dir "$BASE_DIR/smart-contracts/utils/src"

create_dir "$BASE_DIR/backend"
create_dir "$BASE_DIR/backend/eliza_os"
create_dir "$BASE_DIR/backend/api"
create_dir "$BASE_DIR/backend/api/routes"
create_dir "$BASE_DIR/backend/api/models"
create_dir "$BASE_DIR/backend/api/services"
create_dir "$BASE_DIR/backend/oracle"
create_dir "$BASE_DIR/backend/config"
create_dir "$BASE_DIR/backend/utils"
create_dir "$BASE_DIR/backend/tests"

create_dir "$BASE_DIR/frontend/public/assets/images"
create_dir "$BASE_DIR/frontend/src/components/common"
create_dir "$BASE_DIR/frontend/src/components/dashboard"
create_dir "$BASE_DIR/frontend/src/components/loans"
create_dir "$BASE_DIR/frontend/src/components/pools"
create_dir "$BASE_DIR/frontend/src/pages"
create_dir "$BASE_DIR/frontend/src/hooks"
create_dir "$BASE_DIR/frontend/src/services"
create_dir "$BASE_DIR/frontend/src/utils"
create_dir "$BASE_DIR/frontend/src/contexts"
create_dir "$BASE_DIR/frontend/src/types"
create_dir "$BASE_DIR/frontend/src/assets/styles"
create_dir "$BASE_DIR/frontend/src/assets/icons"

create_dir "$BASE_DIR/scripts"
create_dir "$BASE_DIR/docs/contract-abi"
create_dir "$BASE_DIR/docs/diagrams"
create_dir "$BASE_DIR/.github/workflows"

# Criar arquivos de configuração na raiz
cat > "$BASE_DIR/docker-compose.yml" << 'EOF'
version: '3.8'

services:
  # Backend Python API
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    container_name: social-fi-backend
    restart: unless-stopped
    ports:
      - "8000:8000"
    volumes:
      - ./backend:/app
    environment:
      - ENVIRONMENT=development
      - DEBUG=true
      - API_HOST=0.0.0.0
      - API_PORT=8000
      - CORS_ORIGINS=["http://localhost:3000"]
      - CHAIN_ID=D
      - GATEWAY_URL=https://devnet-gateway.multiversx.com
      - CONTRACTS_REPUTATION_SCORE=${REPUTATION_SCORE_ADDRESS}
      - CONTRACTS_LOAN_CONTROLLER=${LOAN_CONTROLLER_ADDRESS}
      - CONTRACTS_LIQUIDITY_POOL=${LIQUIDITY_POOL_ADDRESS}
      - CONTRACTS_DEBT_TOKEN=${DEBT_TOKEN_ADDRESS}
      - TWITTER_API_KEY=${TWITTER_API_KEY}
      - TWITTER_API_SECRET=${TWITTER_API_SECRET}
      - TWITTER_ACCESS_TOKEN=${TWITTER_ACCESS_TOKEN}
      - TWITTER_ACCESS_SECRET=${TWITTER_ACCESS_SECRET}
    networks:
      - social-fi-network

  # ElizaOS Twitter Monitor
  eliza-monitor:
    build:
      context: ./backend
      dockerfile: Dockerfile.eliza
    container_name: social-fi-eliza
    restart: unless-stopped
    volumes:
      - ./backend:/app
    environment:
      - ENVIRONMENT=development
      - DEBUG=true
      - TWITTER_API_KEY=${TWITTER_API_KEY}
      - TWITTER_API_SECRET=${TWITTER_API_SECRET}
      - TWITTER_ACCESS_TOKEN=${TWITTER_ACCESS_TOKEN}
      - TWITTER_ACCESS_SECRET=${TWITTER_ACCESS_SECRET}
      - MONITOR_INTERVAL=900  # 15 minutes in seconds
      - HASHTAG=ElizaOS
      - API_URL=http://backend:8000
    depends_on:
      - backend
    networks:
      - social-fi-network

  # Frontend React app
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    container_name: social-fi-frontend
    restart: unless-stopped
    ports:
      - "3000:80"
    environment:
      - REACT_APP_API_URL=http://localhost:8000
      - REACT_APP_CHAIN_ID=D
      - REACT_APP_ENVIRONMENT=development
    depends_on:
      - backend
    networks:
      - social-fi-network

networks:
  social-fi-network:
    driver: bridge
EOF

cat > "$BASE_DIR/.env.example" << 'EOF'
# MultiversX Contract Addresses
REPUTATION_SCORE_ADDRESS=erd1...
LOAN_CONTROLLER_ADDRESS=erd1...
LIQUIDITY_POOL_ADDRESS=erd1...
DEBT_TOKEN_ADDRESS=erd1...

# Twitter API Credentials
TWITTER_API_KEY=your_twitter_api_key
TWITTER_API_SECRET=your_twitter_api_secret
TWITTER_ACCESS_TOKEN=your_twitter_access_token
TWITTER_ACCESS_SECRET=your_twitter_access_secret
EOF

cat > "$BASE_DIR/LICENSE" << 'EOF'
MIT License

Copyright (c) 2025 social-fi Credit

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF

# Criar arquivos do smart-contract
cat > "$BASE_DIR/smart-contracts/reputation-score/Cargo.toml" << 'EOF'
[package]
name = "reputation-score"
version = "0.1.0"
authors = ["social-fi Credit Team"]
edition = "2021"

[lib]
path = "src/lib.rs"

[dependencies]
multiversx-sc = "0.39.5"

[dev-dependencies]
multiversx-sc-scenario = "0.39.5"
EOF

cat > "$BASE_DIR/smart-contracts/reputation-score/src/lib.rs" << 'EOF'
#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ReputationScore {
    #[init]
    fn init(&self, min_score: u64, max_score: u64) {
        self.min_score().set(min_score);
        self.max_score().set(max_score);
    }

    // Sets or updates the social score for a user
    #[endpoint]
    fn update_score(&self, user_address: ManagedAddress, score: u64) -> SCResult<()> {
        // Only oracle can update scores
        require!(
            self.blockchain().get_caller() == self.oracle_address().get(),
            "Only oracle can update scores"
        );
        
        // Validate score range
        require!(
            score >= self.min_score().get() && score <= self.max_score().get(),
            "Score out of valid range"
        );
        
        self.user_score(user_address).set(score);
        
        Ok(())
    }
    
    // Get user's current score
    #[view(getUserScore)]
    fn get_user_score(&self, user_address: ManagedAddress) -> u64 {
        let default_score = self.min_score().get();
        if self.user_score(user_address).is_empty() {
            return default_score;
        }
        
        self.user_score(user_address).get()
    }
    
    // Check if user is eligible for loan based on minimum score
    #[view(isEligibleForLoan)]
    fn is_eligible_for_loan(&self, user_address: ManagedAddress, required_score: u64) -> bool {
        let user_score = self.get_user_score(user_address);
        user_score >= required_score
    }
    
    // Calculate max loan amount based on user score
    #[view(calculateMaxLoanAmount)]
    fn calculate_max_loan_amount(&self, user_address: ManagedAddress, base_amount: BigUint) -> BigUint {
        let user_score = self.get_user_score(user_address);
        let max_score = self.max_score().get();
        
        // Simple formula: base_amount * (user_score / max_score) * 2
        let user_score_big = BigUint::from(user_score);
        let max_score_big = BigUint::from(max_score);
        
        base_amount * user_score_big * 2u32 / max_score_big
    }
    
    // Storage
    #[storage_mapper("oracle_address")]
    fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;
    
    #[storage_mapper("min_score")]
    fn min_score(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("max_score")]
    fn max_score(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("user_score")]
    fn user_score(&self, user_address: ManagedAddress) -> SingleValueMapper<u64>;
}
EOF

cat > "$BASE_DIR/smart-contracts/loan-controller/Cargo.toml" << 'EOF'
[package]
name = "loan-controller"
version = "0.1.0"
authors = ["social-fi Credit Team"]
edition = "2021"

[lib]
path = "src/lib.rs"

[dependencies]
multiversx-sc = "0.39.5"
reputation-score = { path = "../reputation-score" }

[dev-dependencies]
multiversx-sc-scenario = "0.39.5"
EOF

cat > "$BASE_DIR/smart-contracts/loan-controller/src/lib.rs" << 'EOF'
#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait LoanController {
    #[init]
    fn init(
        &self,
        reputation_score_address: ManagedAddress,
        min_required_score: u64,
        interest_rate_base: u64,
    ) {
        self.reputation_score_address().set(reputation_score_address);
        self.min_required_score().set(min_required_score);
        self.interest_rate_base().set(interest_rate_base);
    }

    // Request a loan
    #[payable("*")]
    #[endpoint(requestLoan)]
    fn request_loan(&self, amount: BigUint, duration_days: u64) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        // Check if user has sufficient score
        let rs_proxy = self.reputation_score_proxy(self.reputation_score_address().get());
        require!(
            rs_proxy.is_eligible_for_loan(caller.clone(), self.min_required_score().get()),
            "User score too low for loan"
        );
        
        // Check if amount is within user's limit
        let max_amount = rs_proxy.calculate_max_loan_amount(caller.clone(), self.base_loan_amount().get());
        require!(
            amount <= max_amount,
            "Requested amount exceeds maximum allowed"
        );
        
        // Calculate interest rate based on user's score
        let user_score = rs_proxy.get_user_score(caller.clone());
        let interest_rate = self.calculate_interest_rate(user_score);
        
        // Calculate repayment amount
        let interest_amount = &amount * &BigUint::from(interest_rate) / &BigUint::from(10000u32);
        let repayment_amount = &amount + &interest_amount;
        
        // Create loan
        let loan_id = self.loan_counter().get();
        self.loan_counter().set(loan_id + 1);
        
        let current_timestamp = self.blockchain().get_block_timestamp();
        let due_timestamp = current_timestamp + duration_days * 86400; // 86400 seconds = 1 day
        
        self.loans(loan_id).set(Loan {
            borrower: caller.clone(),
            amount: amount.clone(),
            repayment_amount,
            interest_rate,
            creation_timestamp: current_timestamp,
            due_timestamp,
            status: LoanStatus::Active,
        });
        
        // Associate loan with user
        self.user_loans(caller).push(&loan_id);
        
        // Transfer funds to user
        let token_id = self.call_value().egld_or_single_esdt().token_identifier;
        self.send().direct(&caller, &token_id, 0, &amount);
        
        Ok(())
    }
    
    // Repay a loan
    #[payable("*")]
    #[endpoint(repayLoan)]
    fn repay_loan(&self, loan_id: u64) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        require!(
            !self.loans(loan_id).is_empty(),
            "Loan does not exist"
        );
        
        let mut loan = self.loans(loan_id).get();
        
        require!(
            loan.borrower == caller,
            "Only borrower can repay loan"
        );
        
        require!(
            loan.status == LoanStatus::Active,
            "Loan is not active"
        );
        
        // Check payment amount
        let payment = self.call_value().egld_or_single_esdt();
        require!(
            payment.amount == loan.repayment_amount,
            "Payment amount does not match repayment amount"
        );
        
        // Update loan status
        loan.status = LoanStatus::Repaid;
        self.loans(loan_id).set(loan);
        
        // Update user score if paid on time
        let current_timestamp = self.blockchain().get_block_timestamp();
        if current_timestamp <= loan.due_timestamp {
            // Positive score update would be triggered by the oracle
            // but we could record the on-time payment
            self.on_time_payments(caller).update(|count| *count += 1);
        }
        
        // Transfer funds to liquidity pool
        // In a real implementation, this would distribute to investors
        
        Ok(())
    }
    
    // Calculate interest rate based on user score
    fn calculate_interest_rate(&self, user_score: u64) -> u64 {
        let base_rate = self.interest_rate_base().get();
        let max_score = 1000u64; // Assume max score is 1000
        
        // Formula: base_rate * (1 - (user_score / max_score) * 0.8)
        // This means higher score = lower interest rate
        // E.g., if base_rate = 1000 (10%), a max score user would pay 2%
        
        let score_factor = (user_score * 80) / max_score; // 0-80 range
        if score_factor >= 100 { 
            return base_rate / 5; // Minimum 20% of base rate
        }
        
        base_rate * (100 - score_factor) / 100
    }
    
    // Storage
    #[storage_mapper("reputation_score_address")]
    fn reputation_score_address(&self) -> SingleValueMapper<ManagedAddress>;
    
    #[storage_mapper("min_required_score")]
    fn min_required_score(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("interest_rate_base")]
    fn interest_rate_base(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("base_loan_amount")]
    fn base_loan_amount(&self) -> SingleValueMapper<BigUint>;
    
    #[storage_mapper("loan_counter")]
    fn loan_counter(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("loans")]
    fn loans(&self, loan_id: u64) -> SingleValueMapper<Loan<Self::Api>>;
    
    #[storage_mapper("user_loans")]
    fn user_loans(&self, user: ManagedAddress) -> VecMapper<u64>;
    
    #[storage_mapper("on_time_payments")]
    fn on_time_payments(&self, user: ManagedAddress) -> SingleValueMapper<u64>;
    
    // Proxy to reputation score contract
    #[proxy]
    fn reputation_score_proxy(&self, address: ManagedAddress) -> reputation_score::Proxy<Self::Api>;
}

// Loan status enum
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq)]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
}

// Loan struct
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Loan<M: ManagedTypeApi> {
    pub borrower: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub repayment_amount: BigUint<M>,
    pub interest_rate: u64,
    pub creation_timestamp: u64,
    pub due_timestamp: u64,
    pub status: LoanStatus,
}
EOF

# Criar backend
cat > "$BASE_DIR/backend/requirements.txt" << 'EOF'
fastapi==0.95.0
uvicorn==0.21.1
aiohttp==3.8.4
tweepy==4.14.0
pydantic==1.10.7
pytextrank==3.2.4
spacy==3.5.1
scikit-learn==1.2.2
python-jose==3.3.0
python-multipart==0.0.6
celery==5.2.7
redis==4.5.4
pytest==7.3.1
pytest-asyncio==0.21.0
EOF

cat > "$BASE_DIR/backend/Dockerfile" << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Set environment variables
ENV PYTHONPATH=/app
ENV PYTHONUNBUFFERED=1

# Run the application
CMD ["python", "-m", "api.main"]
EOF

cat > "$BASE_DIR/backend/config/settings.py" << 'EOF'
import os
import json
from typing import List, Dict, Any, Optional

class Settings:
    """Application settings"""
    
    # Environment
    ENVIRONMENT: str = os.getenv("ENVIRONMENT", "development")
    DEBUG: bool = os.getenv("DEBUG", "true").lower() == "true"
    
    # API Settings
    API_HOST: str = os.getenv("API_HOST", "0.0.0.0")
    API_PORT: int = int(os.getenv("API_PORT", "8000"))
    
    # CORS
    CORS_ORIGINS: List[str] = json.loads(os.getenv("CORS_ORIGINS", '["http://localhost:3000"]'))
    
    # MultiversX Blockchain
    CHAIN_ID: str = os.getenv("CHAIN_ID", "D")  # D for devnet
    GATEWAY_URL: str = os.getenv("GATEWAY_URL", "https://devnet-gateway.multiversx.com")
    
    # Smart Contract Addresses
    CONTRACTS: Dict[str, str] = {
        "REPUTATION_SCORE": os.getenv("CONTRACTS_REPUTATION_SCORE", ""),
        "LOAN_CONTROLLER": os.getenv("CONTRACTS_LOAN_CONTROLLER", ""),
        "LIQUIDITY_POOL": os.getenv("CONTRACTS_LIQUIDITY_POOL", ""),
        "DEBT_TOKEN": os.getenv("CONTRACTS_DEBT_TOKEN", ""),
    }
    
    # Twitter API
    TWITTER_API_KEY: str = os.getenv("TWITTER_API_KEY", "")
    TWITTER_API_SECRET: str = os.getenv("TWITTER_API_SECRET", "")
    TWITTER_ACCESS_TOKEN: str = os.getenv("TWITTER_ACCESS_TOKEN", "")
    TWITTER_ACCESS_SECRET: str = os.getenv("TWITTER_ACCESS_SECRET", "")
    
    # ElizaOS Settings
    MONITOR_INTERVAL: int = int(os.getenv("MONITOR_INTERVAL", "900"))  # 15 minutes in seconds
    HASHTAG: str = os.getenv("HASHTAG", "ElizaOS")

# Singleton instance
settings = Settings()
EOF

cat > "$BASE_DIR/backend/config/__init__.py" << 'EOF'
# Config package
EOF

cat > "$BASE_DIR/backend/eliza_os/__init__.py" << 'EOF'
# ElizaOS package
EOF

cat > "$BASE_DIR/backend/eliza_os/twitter_monitor.py" << 'EOF'
import tweepy
import logging
from datetime import datetime, timedelta
from typing import List, Dict, Any

logger = logging.getLogger(__name__)

class TwitterMonitor:
    """
    Monitors Twitter/X for interactions with the #ElizaOS hashtag
    """
    
    def __init__(self, api_key: str, api_secret: str, access_token: str, access_token_secret: str):
        """Initialize the Twitter API client"""
        auth = tweepy.OAuth1UserHandler(
            api_key, api_secret, access_token, access_token_secret
        )
        self.api = tweepy.API(auth)
        self.client = tweepy.Client(
            consumer_key=api_key,
            consumer_secret=api_secret,
            access_token=access_token,
            access_token_secret=access_token_secret
        )
        
    def search_recent_mentions(self, hashtag: str = "ElizaOS", hours: int = 24) -> List[Dict[Any, Any]]:
        """
        Search for tweets containing the specified hashtag in the last X hours
        
        Args:
            hashtag: The hashtag to search for (without #)
            hours: How many hours back to search
            
        Returns:
            List of tweet data
        """
        since_time = datetime.utcnow() - timedelta(hours=hours)
        query = f"#{hashtag} -is:retweet"
        
        try:
            tweets = self.client.search_recent_tweets(
                query=query,
                max_results=100,
                tweet_fields=['created_at', 'public_metrics', 'author_id', 'text']
            )
            
            if not tweets.data:
                logger.info(f"No tweets found with #{hashtag} in the last {hours} hours")
                return []
                
            tweet_data = []
            for tweet in tweets.data:
                tweet_data.append({
                    'id': tweet.id,
                    'text': tweet.text,
                    'created_at': tweet.created_at,
                    'author_id': tweet.author_id,
                    'retweet_count': tweet.public_metrics['retweet_count'],
                    'reply_count': tweet.public_metrics['reply_count'],
                    'like_count': tweet.public_metrics['like_count'],
                    'quote_count': tweet.public_metrics['quote_count']
                })
                
            return tweet_data
            
        except Exception as e:
            logger.error(f"Error searching for tweets: {e}")
            return []
            
    def get_user_by_username(self, username: str) -> Dict[Any, Any]:
        """
        Get user information by username
        
        Args:
            username: Twitter username without @
            
        Returns:
            User data or None if not found
        """
        try:
            user = self.client.get_user(
                username=username,
                user_fields=['id', 'name', 'username', 'public_metrics']
            )
            
            if not user.data:
                return None
                
            return {
                'id': user.data.id,
                'name': user.data.name,
                'username': user.data.username,
                'followers_count': user.data.public_metrics['followers_count'],
                'following_count': user.data.public_metrics['following_count'],
                'tweet_count': user.data.public_metrics['tweet_count']
            }
            
        except Exception as e:
            logger.error(f"Error getting user: {e}")
            return None
    
    def send_direct_message(self, recipient_id: str, message: str) -> bool:
        """
        Send a direct message to a user
        
        Args:
            recipient_id: Twitter user ID to send message to
            message: Message text
            
        Returns:
            True if successful, False otherwise
        """
        try:
            self.client.create_direct_message(
                participant_id=recipient_id,
                text=message
            )
            return True
        except Exception as e:
            logger.error(f"Error sending DM: {e}")
            return False
EOF

cat > "$BASE_DIR/backend/eliza_os/sentiment_analyzer.py" << 'EOF'
import logging
from typing import Any
import re
import math
from sklearn.feature_extraction.text import CountVectorizer
from sklearn.naive_bayes import MultinomialNB
import spacy
import pytextrank

logger = logging.getLogger(__name__)

class SentimentAnalyzer:
    """
    Analyzes sentiment of text content related to social-fi Credit
    """
    
    def __init__(self):
        """Initialize the sentiment analyzer"""
        # Load SpaCy model
        try:
            self.nlp = spacy.load("en_core_web_sm")
            # Add TextRank component to pipeline
            self.nlp.add_pipe("textrank")
        except Exception as e:
            logger.error(f"Error loading NLP model: {e}")
            # Fallback to simple model
            self.nlp = None
            
        # Initialize classifier
        self.vectorizer = CountVectorizer(stop_words='english')
        self.classifier = MultinomialNB()
        
        # Train on sample data
        self._train_classifier()
        
    def _train_classifier(self):
        """Train the sentiment classifier on sample data"""
        # Sample training data
        texts = [
            "Great project, very innovative!", 
            "Loving the social-fi Credit system",
            "This is revolutionary for DeFi",
            "Amazing work with the zero-collateral loans",
            "Really impressed with ElizaOS integration",
            "The community score system is brilliant",
            "Excellent response to my question!",
            "Very helpful team",
            "This project is going to change everything",
            "Super excited about this",
            "This doesn't work at all",
            "Terrible experience with the loans",
            "Not impressed with the scoring system",
            "Too complicated to use",
            "Failed to get a loan despite high score",
            "Bug in the system, lost my money",
            "Poor documentation",
            "This is a scam",
            "Waste of time",
            "The team doesn't respond to questions"
        ]
        
        labels = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]  # 1 for positive, 0 for negative
        
        # Train the classifier
        try:
            X = self.vectorizer.fit_transform(texts)
            self.classifier.fit(X, labels)
            logger.info("Sentiment classifier trained successfully")
        except Exception as e:
            logger.error(f"Error training classifier: {e}")
        
    def analyze_text(self, text: str) -> float:
        """
        Analyze text sentiment and return a score from -1.0 to 1.0
        
        Args:
            text: Text to analyze
            
        Returns:
            Sentiment score (-1.0 to 1.0)
        """
        if not text:
            return 0.0
            
        # Clean text
        text = self._clean_text(text)
        
        # Get basic sentiment score from classifier
        basic_score = self._classify_sentiment(text)
        
        # Enhance with NLP analysis if available
        enhanced_score = basic_score
        if self.nlp:
            enhanced_score = self._enhance_sentiment(text, basic_score)
            
        return enhanced_score
    def _clean_text(self, text: str) -> str:
        """Clean and normalize text"""
        # Remove URLs
        text = re.sub(r'https?://\S+', '', text)
        
        # Remove user mentions
        text = re.sub(r'@\w+', '', text)
        
        # Remove hashtags but keep the text
        text = re.sub(r'#(\w+)', r'\1', text)
        
        # Remove extra whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        
        return text
        
    def _classify_sentiment(self, text: str) -> float:
        """Use the trained classifier to get sentiment"""
        try:
            X = self.vectorizer.transform([text])
            # Get probability of positive class
            prob = self.classifier.predict_proba(X)[0][1]
            
            # Convert to -1.0 to 1.0 scale
            return (prob * 2) - 1.0
        except Exception as e:
            logger.error(f"Error classifying sentiment: {e}")
            return 0.0
            
    def _enhance_sentiment(self, text: str, basic_score: float) -> float:
        """Enhance sentiment analysis with NLP techniques"""
        try:
            doc = self.nlp(text)
            
            # Get keywords
            keywords = [phrase.text for phrase in doc._.phrases[:5]]
            
            # Check for sentiment modifiers
            intensifiers = ['very', 'super', 'really', 'extremely', 'absolutely']
            diminishers = ['somewhat', 'slightly', 'a bit', 'kind of', 'sort of']
            
            # Adjust score based on modifiers
            modifier = 1.0
            for intensifier in intensifiers:
                if intensifier in text.lower():
                    modifier *= 1.2
                    
            for diminisher in diminishers:
                if diminisher in text.lower():
                    modifier *= 0.8
                    
            # Ensure score stays in range
            enhanced_score = max(min(basic_score * modifier, 1.0), -1.0)
            
            return enhanced_score
            
        except Exception as e:
            logger.error(f"Error enhancing sentiment: {e}")
            return basic_score
EOF

cat > "$BASE_DIR/backend/eliza_os/score_calculator.py" << 'EOF'
from typing import Dict, List, Any
import logging
import math
from datetime import datetime

logger = logging.getLogger(__name__)

class ScoreCalculator:
    """
    Calculates user Community Score based on social media interactions
    """
    
    def __init__(self, sentiment_analyzer):
        """Initialize with sentiment analyzer"""
        self.sentiment_analyzer = sentiment_analyzer
        
        # Score multipliers for different types of interactions
        self.score_weights = {
            'positive_mention': 5,      # Mentioning project positively
            'technical_answer': 10,     # Answering technical questions
            'resource_sharing': 7,      # Sharing tutorials/resources
            'like_received': 1,         # Getting likes on project-related tweets
            'retweet_received': 3,      # Getting retweets on project-related tweets
            'follower_factor': 0.1      # Follower count factor (0.1 * log(followers))
        }
        
    def calculate_user_score(self, user_data: Dict[str, Any], tweets: List[Dict[str, Any]]) -> int:
        """
        Calculate a user's community score based on their activity
        
        Args:
            user_data: User profile data including follower counts
            tweets: List of tweet data from the user
            
        Returns:
            Calculated score as integer
        """
        if not user_data or not tweets:
            return 0
            
        total_score = 0
        
        # Base score from user metrics (followers)
        follower_count = user_data.get('followers_count', 0)
        follower_score = 0
        if follower_count > 0:
            import math
            follower_score = int(self.score_weights['follower_factor'] * math.log(follower_count + 1))
        
        total_score += follower_score
        
        # Analyze tweets
        for tweet in tweets:
            tweet_score = 0
            
            # Analyze sentiment
            sentiment = self.sentiment_analyzer.analyze_text(tweet['text'])
            
            # Score based on content type
            if self._is_positive_mention(tweet['text'], sentiment):
                tweet_score += self.score_weights['positive_mention']
                
            if self._is_technical_answer(tweet['text']):
                tweet_score += self.score_weights['technical_answer']
                
            if self._is_resource_sharing(tweet['text']):
                tweet_score += self.score_weights['resource_sharing']
                
            # Score based on engagement metrics
            tweet_score += tweet.get('like_count', 0) * self.score_weights['like_received']
            tweet_score += tweet.get('retweet_count', 0) * self.score_weights['retweet_received']
            
            # Apply time decay factor (more recent = more value)
            tweet_score = self._apply_time_decay(tweet_score, tweet['created_at'])
            
            total_score += tweet_score
            
        # Round to integer
        return int(total_score)
        
    def _is_positive_mention(self, text: str, sentiment: float) -> bool:
        """Detect if text is a positive project mention"""
        project_keywords = ['social-fi', 'credit', 'social-ficredit', 'elizaos']
        return any(keyword in text.lower() for keyword in project_keywords) and sentiment > 0.2
        
    def _is_technical_answer(self, text: str) -> bool:
        """Detect if text is answering a technical question"""
        technical_indicators = ['how to', 'problem', 'error', 'issue', 'fix', 'solution', 'code', 'contract']
        return any(indicator in text.lower() for indicator in technical_indicators) and len(text) > 100
        
    def _is_resource_sharing(self, text: str) -> bool:
        """Detect if text is sharing resources or tutorials"""
        resource_indicators = ['guide', 'tutorial', 'documentation', 'learn', 'http', 'https', 'github']
        return any(indicator in text.lower() for indicator in resource_indicators)
        
    def _apply_time_decay(self, score: float, created_at: datetime) -> float:
        """Apply time decay to score - more recent tweets worth more"""
        days_ago = (datetime.utcnow() - created_at).days
        if days_ago <= 1:
            return score  # Full value for tweets within last day
        elif days_ago <= 7:
            return score * 0.8  # 80% value for tweets within last week
        elif days_ago <= 30:
            return score * 0.5  # 50% value for tweets within last month
        else:
            return score * 0.2  # 20% value for older tweets
EOF

cat > "$BASE_DIR/backend/eliza_os/main.py" << 'EOF'
import logging
import asyncio
import time
from datetime import datetime, timedelta
import json
from typing import Dict, List, Any

from .twitter_monitor import TwitterMonitor
from .sentiment_analyzer import SentimentAnalyzer
from .score_calculator import ScoreCalculator
from config.settings import settings

# Configure logging
logging.basicConfig(
    level=logging.INFO if not settings.DEBUG else logging.DEBUG,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

class ElizaOS:
    """
    Main ElizaOS system that monitors social media interactions and updates user scores
    """
    
    def __init__(self):
        """Initialize ElizaOS components"""
        # Check if Twitter API credentials are available
        if not all([
            settings.TWITTER_API_KEY,
            settings.TWITTER_API_SECRET,
            settings.TWITTER_ACCESS_TOKEN,
            settings.TWITTER_ACCESS_SECRET
        ]):
            logger.error("Twitter API credentials not configured")
            raise ValueError("Twitter API credentials are required")
            
        # Initialize components
        self.twitter_monitor = TwitterMonitor(
            api_key=settings.TWITTER_API_KEY,
            api_secret=settings.TWITTER_API_SECRET,
            access_token=settings.TWITTER_ACCESS_TOKEN,
            access_token_secret=settings.TWITTER_ACCESS_SECRET
        )
        
        self.sentiment_analyzer = SentimentAnalyzer()
        self.score_calculator = ScoreCalculator(self.sentiment_analyzer)
        
        # Settings
        self.monitor_interval = settings.MONITOR_INTERVAL
        self.hashtag = settings.HASHTAG
        
        # State
        self.user_data = {}
        self.last_update = datetime.utcnow() - timedelta(days=1)  # Force initial update
        
    async def start(self):
        """Start the ElizaOS monitor service"""
        logger.info(f"Starting ElizaOS monitor service (checking #{self.hashtag} every {self.monitor_interval} seconds)")
        
        while True:
            try:
                # Check if it's time to update
                current_time = datetime.utcnow()
                time_diff = (current_time - self.last_update).total_seconds()
                
                if time_diff >= self.monitor_interval:
                    logger.info(f"Updating social scores (last update: {self.last_update.isoformat()})")
                    
                    # Search for tweets with the hashtag
                    tweets = await self.search_recent_interactions()
                    
                    if tweets:
                        # Process tweets and update scores
                        await self.process_interactions(tweets)
                        
                    self.last_update = current_time
                    
                # Sleep until next check
                await asyncio.sleep(60)  # Check every minute
                
            except Exception as e:
                logger.error(f"Error in ElizaOS monitor: {e}")
                await asyncio.sleep(300)  # Wait 5 minutes if there's an error
        
    async def search_recent_interactions(self) -> List[Dict[str, Any]]:
        """Search for recent interactions with the hashtag"""
        try:
            # Get tweets from last interval
            hours = self.monitor_interval / 3600 + 1  # Add 1 hour buffer
            tweets = self.twitter_monitor.search_recent_mentions(
                hashtag=self.hashtag,
                hours=hours
            )
            
            logger.info(f"Found {len(tweets)} tweets with #{self.hashtag}")
            return tweets
            
        except Exception as e:
            logger.error(f"Error searching for interactions: {e}")
            return []
            
    async def process_interactions(self, tweets: List[Dict[str, Any]]):
        """Process tweets and update user scores"""
        # Group tweets by author
        tweets_by_author = {}
        for tweet in tweets:
            author_id = tweet.get('author_id')
            if author_id:
                if author_id not in tweets_by_author:
                    tweets_by_author[author_id] = []
                tweets_by_author[author_id].append(tweet)
                
        # Update scores for each user
        for author_id, user_tweets in tweets_by_author.items():
            try:
                # Get user data
                user_data = await self.get_user_data(author_id)
                if not user_data:
                    continue
                    
                # Calculate score
                score = self.score_calculator.calculate_user_score(user_data, user_tweets)
                
                # Update score in the blockchain
                await self.update_user_score(user_data, score)
                
                # Send notification if score changed significantly
                if user_data.get('address'):
                    await self.send_score_notification(user_data, score)
                    
            except Exception as e:
                logger.error(f"Error processing user {author_id}: {e}")
                
    async def get_user_data(self, twitter_id: str) -> Dict[str, Any]:
        """Get user data from API"""
        # In a real implementation, this would call the backend API
        # For now, just return mock data
        return {
            "id": twitter_id,
            "username": f"user{twitter_id}",
            "address": f"erd1_mock_address_{twitter_id}",
            "followers_count": 150,
            "following_count": 120,
            "tweet_count": 450
        }
        
    async def update_user_score(self, user_data: Dict[str, Any], score: int):
        """Update user's score in the blockchain"""
        # In a real implementation, this would call the blockchain service
        logger.info(f"Updating score for user {user_data['username']} to {score}")
        
        # Save to local state for now
        self.user_data[user_data['id']] = {
            **user_data,
            "score": score,
            "updated_at": datetime.utcnow().isoformat()
        }
        
    async def send_score_notification(self, user_data: Dict[str, Any], score: int):
        """Send notification about score update"""
        # Get previous score
        previous_data = self.user_data.get(user_data['id'], {})
        previous_score = previous_data.get('score', 0)
        
        # Check if score changed significantly (more than 10%)
        if previous_score > 0 and abs(score - previous_score) / previous_score >= 0.1:
            difference = score - previous_score
            
            # In a real implementation, this would send a DM via Twitter
            logger.info(f"Sending notification to {user_data['username']} about score change: {difference:+d}")
            
            if difference > 0:
                message = f"Good news! Your social-fi Credit Community Score has increased by {difference} points. You can now borrow more crypto without collateral! Check your profile at social-ficredit.io/profile"
            else:
                message = f"Your social-fi Credit Community Score has decreased by {abs(difference)} points. Continue engaging positively with the community to improve your score. Visit social-ficredit.io/profile for more details."
                
            # This would actually send the DM in a real implementation
            # self.twitter_monitor.send_direct_message(user_data['id'], message)
        
async def main():
    """Main entry point for ElizaOS"""
    eliza = ElizaOS()
    await eliza.start()

if __name__ == "__main__":
    asyncio.run(main())
EOF

# Criar arquivos para a API
cat > "$BASE_DIR/backend/api/__init__.py" << 'EOF'
# API package
EOF

cat > "$BASE_DIR/backend/api/main.py" << 'EOF'
from fastapi import FastAPI, HTTPException, Depends
from fastapi.middleware.cors import CORSMiddleware
from typing import List, Dict, Any, Optional
import uvicorn
import logging

from .routes import users, loans, pools
from .services.blockchain import BlockchainService
from config.settings import settings

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="social-fi Credit API",
    description="Backend API for social-fi Credit DeFi platform",
    version="1.0.0",
)

# Configure CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(users.router, prefix="/api/users", tags=["Users"])
app.include_router(loans.router, prefix="/api/loans", tags=["Loans"])
app.include_router(pools.router, prefix="/api/pools", tags=["Pools"])

# Dependency for blockchain service
def get_blockchain_service() -> BlockchainService:
    return BlockchainService(
        chain_id=settings.CHAIN_ID,
        gateway_url=settings.GATEWAY_URL,
        contracts={
            "reputation_score": settings.CONTRACTS["REPUTATION_SCORE"],
            "loan_controller": settings.CONTRACTS["LOAN_CONTROLLER"],
            "liquidity_pool": settings.CONTRACTS["LIQUIDITY_POOL"],
            "debt_token": settings.CONTRACTS["DEBT_TOKEN"],
        }
    )

# Health check endpoint
@app.get("/health", tags=["Health"])
async def health_check():
    return {
        "status": "ok",
        "version": app.version,
        "environment": settings.ENVIRONMENT,
    }

# Root endpoint
@app.get("/", tags=["Root"])
async def root():
    return {
        "name": "social-fi Credit API",
        "description": "API for zero-collateral DeFi loans based on social reputation",
        "docs": "/docs",
    }

if __name__ == "__main__":
    logger.info(f"Starting social-fi Credit API server on port {settings.API_PORT}")
    uvicorn.run(
        "api.main:app",
        host=settings.API_HOST,
        port=settings.API_PORT,
        reload=settings.DEBUG,
    )
EOF

# Criar routes principais
cat > "$BASE_DIR/backend/api/routes/__init__.py" << 'EOF'
# Routes package
EOF

cat > "$BASE_DIR/backend/api/routes/users.py" << 'EOF'
from fastapi import APIRouter, HTTPException, Depends
from typing import List, Dict, Any, Optional
from pydantic import BaseModel, Field, validator
import logging

from ..services.blockchain import BlockchainService
from ..services.user_service import UserService
from api.main import get_blockchain_service

logger = logging.getLogger(__name__)
router = APIRouter()

# Models
class UserScore(BaseModel):
    current: int = Field(..., description="Current community score")
    max: int = Field(..., description="Maximum possible score")
    eligibleForLoan: bool = Field(..., description="Whether user is eligible for loans")
    maxLoanAmount: str = Field(..., description="Maximum loan amount user can request")

class TwitterConnection(BaseModel):
    twitterHandle: str = Field(..., description="Twitter handle without @")
    oauthToken: str = Field(..., description="OAuth token for Twitter API")

class UserProfile(BaseModel):
    address: str = Field(..., description="Wallet address")
    twitterId: Optional[str] = Field(None, description="Twitter user ID")
    twitterHandle: Optional[str] = Field(None, description="Twitter handle")
    score: int = Field(..., description="Community score")
    loansTaken: int = Field(..., description="Number of loans taken")
    loansRepaid: int = Field(..., description="Number of loans repaid")
    registeredAt: str = Field(..., description="Registration timestamp")

# Endpoints
@router.get("/{address}", response_model=UserProfile)
async def get_user_profile(address: str, blockchain: BlockchainService = Depends(get_blockchain_service)):
    """Get user profile information"""
    try:
        user_service = UserService(blockchain)
        user = await user_service.get_user_info(address)
        
        if not user:
            raise HTTPException(status_code=404, detail="User not found")
            
        return user
    except Exception as e:
        logger.error(f"Error getting user profile: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/{address}/score", response_model=UserScore)
async def get_user_score(address: str, blockchain: BlockchainService = Depends(get_blockchain_service)):
    """Get user's community score and loan eligibility"""
    try:
        user_service = UserService(blockchain)
        score = await user_service.get_score(address)
        
        if not score:
            raise HTTPException(status_code=404, detail="Score not found")
            
        return score
    except Exception as e:
        logger.error(f"Error getting user score: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/{address}/connect-twitter")
async def connect_twitter(
    address: str, 
    twitter_data: TwitterConnection, 
    blockchain: BlockchainService = Depends(get_blockchain_service)
):
    """Connect user's Twitter account"""
    try:
        user_service = UserService(blockchain)
        success = await user_service.connect_twitter(
            address, 
            twitter_data.twitterHandle, 
            twitter_data.oauthToken
        )
        
        if success:
            return {"status": "success", "message": "Twitter account connected successfully"}
        else:
            raise HTTPException(status_code=400, detail="Failed to connect Twitter account")
    except Exception as e:
        logger.error(f"Error connecting Twitter: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/{address}/twitter-stats")
async def get_twitter_stats(address: str, blockchain: BlockchainService = Depends(get_blockchain_service)):
    """Get user's Twitter activity stats related to social-fi Credit"""
    try:
        user_service = UserService(blockchain)
        stats = await user_service.get_twitter_stats(address)
        
        if not stats:
            raise HTTPException(status_code=404, detail="Twitter stats not found")
            
        return stats
    except Exception as e:
        logger.error(f"Error getting Twitter stats: {e}")
        raise HTTPException(status_code=500, detail=str(e))
EOF

cat > "$BASE_DIR/backend/api/routes/loans.py" << 'EOF'
from fastapi import APIRouter, HTTPException, Depends, Query
from typing import List, Dict, Any, Optional
from pydantic import BaseModel, Field, validator
import logging

from ..services.blockchain import BlockchainService
from ..services.loan_service import LoanService
from api.main import get_blockchain_service

logger = logging.getLogger(__name__)
router = APIRouter()

# Models
class LoanRequest(BaseModel):
    amount: str = Field(..., description="Loan amount requested")
    duration_days: int = Field(..., description="Loan duration in days")
    token_id: str = Field(..., description="Token identifier for the loan")

class Loan(BaseModel):
    id: str = Field(..., description="Loan ID")
    borrower: str = Field(..., description="Borrower address")
    amount: str = Field(..., description="Loan amount")
    repayment_amount: str = Field(..., description="Amount to be repaid")
    interest_rate: float = Field(..., description="Interest rate in percentage")
    created_at: str = Field(..., description="Creation timestamp")
    due_date: str = Field(..., description="Due date timestamp")
    status: str = Field(..., description="Loan status (Active, Repaid, Defaulted)")
    nft_id: Optional[str] = Field(None, description="NFT ID if tokenized")

class RepaymentRequest(BaseModel):
    loan_id: str = Field(..., description="Loan ID to repay")
    token_id: str = Field(..., description="Token identifier for repayment")

# Endpoints
@router.get("/", response_model=List[Loan])
async def get_loans(
    address: Optional[str] = None, 
    status: Optional[str] = None,
    skip: int = 0, 
    limit: int = 10,
    blockchain: BlockchainService = Depends(get_blockchain_service)
):
    """Get loans with optional filtering"""
    try:
        loan_service = LoanService(blockchain)
        loans = await loan_service.get_loans(address, status, skip, limit)
        return loans
    except Exception as e:
        logger.error(f"Error getting loans: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/{loan_id}", response_model=Loan)
async def get_loan(loan_id: str, blockchain: BlockchainService = Depends(get_blockchain_service)):
    """Get loan details by ID"""
    try:
        loan_service = LoanService(blockchain)
        loan = await loan_service.get_loan(loan_id)
        
        if not loan:
            raise HTTPException(status_code=404, detail="Loan not found")
            
        return loan
    except Exception as e:
        logger.error(f"Error getting loan: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/request", status_code=201)
async def request_loan(
    loan_request: LoanRequest, 
    blockchain: BlockchainService = Depends(get_blockchain_service)
):
    """Request a new loan"""
    try:
        loan_service = LoanService(blockchain)
        
        # Create loan request transaction for signing
        tx_data = await loan_service.create_loan_request_transaction(
            amount=loan_request.amount,
            duration_days=loan_request.duration_days,
            token_id=loan_request.token_id
        )
        
        return {
            "status": "success", 
            "message": "Loan request created. Please sign the transaction in your wallet.",
            "transaction": tx_data
        }
    except Exception as e:
        logger.error(f"Error requesting loan: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/repay", status_code=200)
async def repay_loan(
    repayment: RepaymentRequest, 
    blockchain: BlockchainService = Depends(get_blockchain_service)
):
    """Repay an existing loan"""
    try:
        loan_service = LoanService(blockchain)
        
        # Create loan repayment transaction for signing
        tx_data = await loan_service.create_loan_repayment_transaction(
            loan_id=repayment.loan_id,
            token_id=repayment.token_id
        )
        
        return {
            "status": "success", 
            "message": "Loan repayment prepared. Please sign the transaction in your wallet.",
            "transaction": tx_data
        }
    except Exception as e:
        logger.error(f"Error repaying loan: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/calculate-interest")
async def calculate_interest(
    amount: str,
    address: str,
    blockchain: BlockchainService = Depends(get_blockchain_service)
):
    """Calculate interest rate and repayment amount for a user and loan amount"""
    try:
        loan_service = LoanService(blockchain)
        calculation = await loan_service.calculate_loan_interest(amount, address)
        
        return calculation
    except Exception as e:
        logger.error(f"Error calculating interest: {e}")
        raise HTTPException(status_code=500, detail=str(e))
EOF

# Criar services
cat > "$BASE_DIR/backend/api/services/__init__.py" << 'EOF'
# Services package
EOF

cat > "$BASE_DIR/backend/api/services/blockchain.py" << 'EOF'
from typing import Dict, Any, List, Optional
import logging
import aiohttp
import json
from config.settings import settings

logger = logging.getLogger(__name__)

class BlockchainService:
    """Service for interacting with MultiversX blockchain"""
    
    def __init__(self, chain_id: str, gateway_url: str, contracts: Dict[str, str]):
        """Initialize the blockchain service"""
        self.chain_id = chain_id
        self.gateway_url = gateway_url
        self.contracts = contracts
        
    async def get_account(self, address: str) -> Dict[str, Any]:
        """Get account details from blockchain"""
        try:
            async with aiohttp.ClientSession() as session:
                url = f"{self.gateway_url}/address/{address}"
                async with session.get(url) as response:
                    if response.status != 200:
                        logger.error(f"Error fetching account: {response.status}")
                        return None
                    
                    data = await response.json()
                    return data
        except Exception as e:
            logger.error(f"Error in get_account: {e}")
            return None
            
    async def query_contract(self, contract_name: str, function: str, args: List[str] = None) -> Dict[str, Any]:
        """Query smart contract view function"""
        if contract_name not in self.contracts:
            raise ValueError(f"Contract '{contract_name}' not configured")
            
        contract_address = self.contracts[contract_name]
        
        try:
            async with aiohttp.ClientSession() as session:
                url = f"{self.gateway_url}/vm-values/query"
                
                payload = {
                    "scAddress": contract_address,
                    "funcName": function,
                    "args": args or []
                }
                
                async with session.post(url, json=payload) as response:
                    if response.status != 200:
                        logger.error(f"Error querying contract: {response.status}")
                        return None
                    
                    data = await response.json()
                    return data
        except Exception as e:
            logger.error(f"Error in query_contract: {e}")
            return None
            
    async def create_transaction(
        self, 
        sender: str, 
        receiver: str, 
        value: str, 
        data: str,
        gas_limit: int = 50000
    ) -> Dict[str, Any]:
        """Create a blockchain transaction object for signing"""
        try:
            # Get account nonce
            account = await self.get_account(sender)
            if not account:
                return None
                
            nonce = account.get("nonce", 0)
            
            # Prepare transaction
            transaction = {
                "nonce": nonce,
                "value": value,
                "receiver": receiver,
                "sender": sender,
                "gasPrice": 1000000000,
                "gasLimit": gas_limit,
                "data": data,
                "chainID": self.chain_id,
                "version": 1
            }
            
            return transaction
            
        except Exception as e:
            logger.error(f"Error creating transaction: {e}")
            return None
    async def call_contract(
        self, 
        contract_name: str, 
        function: str, 
        args: List[str] = None, 
        caller: str = None,
        value: str = "0"
    ) -> Dict[str, Any]:
        """Create a transaction to call a smart contract function"""
        if contract_name not in self.contracts:
            raise ValueError(f"Contract '{contract_name}' not configured")
            
        contract_address = self.contracts[contract_name]
        
        # Encode function call
        function_call = function
        if args:
            for arg in args:
                function_call += f"@{arg}"
                
        # Convert to hex
        data = function_call.encode().hex()
        
        # Create transaction
        return await self.create_transaction(
            sender=caller,
            receiver=contract_address,
            value=value,
            data=data,
            gas_limit=500000  # Higher gas limit for contract calls
        )
EOF

cat > "$BASE_DIR/backend/api/services/user_service.py" << 'EOF'
from typing import Dict, Any, List, Optional
import logging
import json
import base64
from datetime import datetime

from .blockchain import BlockchainService

logger = logging.getLogger(__name__)

class UserService:
    """Service for user-related operations"""
    
    def __init__(self, blockchain: BlockchainService):
        self.blockchain = blockchain
        
    async def get_user_info(self, address: str) -> Dict[str, Any]:
        """Get user profile information"""
        # Query the blockchain for user data
        result = await self.blockchain.query_contract(
            contract_name="reputation_score",
            function="getUserInfo",
            args=[address]
        )
        
        if not result or "returnData" not in result:
            # Create default profile for new users
            return {
                "address": address,
                "twitterId": None,
                "twitterHandle": None,
                "score": 0,
                "loansTaken": 0,
                "loansRepaid": 0,
                "registeredAt": datetime.utcnow().isoformat()
            }
            
        # Decode return data
        try:
            # MultiversX returns base64 encoded data
            decoded_data = []
            for item in result["returnData"]:
                if item:
                    decoded = base64.b64decode(item).decode('utf-8')
                    decoded_data.append(decoded)
                else:
                    decoded_data.append(None)
                    
            # Assuming return format: [twitterId, twitterHandle, score, loansTaken, loansRepaid, timestamp]
            if len(decoded_data) >= 6:
                return {
                    "address": address,
                    "twitterId": decoded_data[0],
                    "twitterHandle": decoded_data[1],
                    "score": int(decoded_data[2] or 0),
                    "loansTaken": int(decoded_data[3] or 0),
                    "loansRepaid": int(decoded_data[4] or 0),
                    "registeredAt": datetime.fromtimestamp(int(decoded_data[5] or 0)).isoformat()
                }
            else:
                # Return default if data structure doesn't match
                return {
                    "address": address,
                    "twitterId": None,
                    "twitterHandle": None,
                    "score": 0,
                    "loansTaken": 0,
                    "loansRepaid": 0,
                    "registeredAt": datetime.utcnow().isoformat()
                }
                
        except Exception as e:
            logger.error(f"Error decoding user data: {e}")
            return None
            
    async def get_score(self, address: str) -> Dict[str, Any]:
        """Get user's community score and loan eligibility"""
        # Query score from blockchain
        result = await self.blockchain.query_contract(
            contract_name="reputation_score",
            function="getUserScore",
            args=[address]
        )
        
        if not result or "returnData" not in result:
            return {
                "current": 0,
                "max": 1000,
                "eligibleForLoan": False,
                "maxLoanAmount": "0"
            }
            
        # Get eligibility
        eligibility = await self.blockchain.query_contract(
            contract_name="reputation_score",
            function="isEligibleForLoan",
            args=[address, "50"]  # Assuming 50 is minimum required score
        )
        
        # Get max loan amount
        loan_amount = await self.blockchain.query_contract(
            contract_name="reputation_score",
            function="calculateMaxLoanAmount",
            args=[address, "1000000000000000000"]  # 1 EGLD in smallest units
        )
        
        try:
            # Decode score
            score_b64 = result["returnData"][0] if result["returnData"] else ""
            score = int(base64.b64decode(score_b64).decode('utf-8')) if score_b64 else 0
            
            # Decode eligibility (boolean)
            eligible_b64 = eligibility["returnData"][0] if eligibility["returnData"] else ""
            eligible_bytes = base64.b64decode(eligible_b64) if eligible_b64 else b'00'
            is_eligible = eligible_bytes[0] == 1 if eligible_bytes else False
            
            # Decode max loan amount
            amount_b64 = loan_amount["returnData"][0] if loan_amount["returnData"] else ""
            max_amount_wei = int(base64.b64decode(amount_b64).decode('utf-8')) if amount_b64 else 0
            max_amount_egld = max_amount_wei / 1000000000000000000  # Convert to EGLD
            
            return {
                "current": score,
                "max": 1000,  # Assuming max score is 1000
                "eligibleForLoan": is_eligible,
                "maxLoanAmount": f"{max_amount_egld:.2f}"
            }
            
        except Exception as e:
            logger.error(f"Error decoding score data: {e}")
            return None
            
    async def connect_twitter(self, address: str, twitter_handle: str, oauth_token: str) -> bool:
        """Connect user's Twitter account"""
        # In a real implementation, we would verify the OAuth token
        # For now, we'll just call the contract with the Twitter handle
        
        try:
            # Encode twitter handle to hex
            handle_hex = twitter_handle.encode().hex()
            
            # Create transaction to update user's Twitter handle
            tx = await self.blockchain.call_contract(
                contract_name="reputation_score",
                function="setTwitterHandle",
                args=[handle_hex],
                caller=address
            )
            
            return tx is not None
            
        except Exception as e:
            logger.error(f"Error connecting Twitter: {e}")
            return False
            
    async def get_twitter_stats(self, address: str) -> Dict[str, Any]:
        """Get user's Twitter activity stats related to social-fi Credit"""
        # This would interact with the Twitter monitoring service
        # For now, return mock data
        user = await self.get_user_info(address)
        
        if not user or not user["twitterHandle"]:
            return None
            
        return {
            "positive_mentions": 12,
            "technical_answers": 3,
            "resources_shared": 5,
            "total_likes_received": 87,
            "total_retweets": 15,
            "last_updated": datetime.utcnow().isoformat()
        }
EOF

# Criar arquivos principais do frontend
cat > "$BASE_DIR/frontend/package.json" << 'EOF'
{
  "name": "social-fi-credit-frontend",
  "version": "0.1.0",
  "private": true,
  "dependencies": {
    "@multiversx/sdk-core": "^12.3.0",
    "@multiversx/sdk-extension-provider": "^2.0.0",
    "@multiversx/sdk-network-providers": "^2.0.0",
    "@multiversx/sdk-wallet-connect-provider": "^3.0.0",
    "@testing-library/jest-dom": "^5.16.5",
    "@testing-library/react": "^14.0.0",
    "@testing-library/user-event": "^14.4.3",
    "@types/jest": "^29.5.1",
    "@types/node": "^18.16.0",
    "@types/react": "^18.0.38",
    "@types/react-dom": "^18.0.11",
    "axios": "^1.3.6",
    "chart.js": "^4.2.1",
    "react": "^18.2.0",
    "react-chartjs-2": "^5.2.0",
    "react-dom": "^18.2.0",
    "react-icons": "^4.8.0",
    "react-router-dom": "^6.10.0",
    "react-scripts": "5.0.1",
    "tailwindcss": "^3.3.1",
    "typescript": "^4.9.5",
    "web-vitals": "^2.1.4"
  },
  "scripts": {
    "start": "react-scripts start",
    "build": "react-scripts build",
    "test": "react-scripts test",
    "eject": "react-scripts eject"
  },
  "eslintConfig": {
    "extends": [
      "react-app",
      "react-app/jest"
    ]
  },
  "browserslist": {
    "production": [
      ">0.2%",
      "not dead",
      "not op_mini all"
    ],
    "development": [
      "last 1 chrome version",
      "last 1 firefox version",
      "last 1 safari version"
    ]
  }
}
EOF

cat > "$BASE_DIR/frontend/tailwind.config.js" << 'EOF'
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#e6f0ff',
          100: '#cce0ff',
          200: '#99c2ff',
          300: '#66a3ff',
          400: '#3385ff',
          500: '#0066ff',
          600: '#0052cc',
          700: '#003d99',
          800: '#002966',
          900: '#001433',
        },
        secondary: {
          50: '#f5e6ff',
          100: '#ebccff',
          200: '#d699ff',
          300: '#c266ff',
          400: '#ad33ff',
          500: '#9900ff',
          600: '#7a00cc',
          700: '#5c0099',
          800: '#3d0066',
          900: '#1f0033',
        },
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
EOF

cat > "$BASE_DIR/frontend/src/index.tsx" << 'EOF'
import React from 'react';
import ReactDOM from 'react-dom/client';
import './assets/styles/global.css';
import App from './App';
import reportWebVitals from './reportWebVitals';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
EOF

cat > "$BASE_DIR/frontend/src/reportWebVitals.ts" << 'EOF'
import { ReportHandler } from 'web-vitals';

const reportWebVitals = (onPerfEntry?: ReportHandler) => {
  if (onPerfEntry && onPerfEntry instanceof Function) {
    import('web-vitals').then(({ getCLS, getFID, getFCP, getLCP, getTTFB }) => {
      getCLS(onPerfEntry);
      getFID(onPerfEntry);
      getFCP(onPerfEntry);
      getLCP(onPerfEntry);
      getTTFB(onPerfEntry);
    });
  }
};

export default reportWebVitals;
EOF

cat > "$BASE_DIR/frontend/src/App.tsx" << 'EOF'
import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { WalletContextProvider } from './contexts/WalletContext';
import { UserContextProvider } from './contexts/UserContext';
import Header from './components/common/Header';
import Footer from './components/common/Footer';
import Home from './pages/Home';
import Dashboard from './pages/Dashboard';
import Loans from './pages/Loans';
import Pools from './pages/Pools';
import Profile from './pages/Profile';
import NFTMarketplace from './pages/NFTMarketplace';
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
EOF

cat > "$BASE_DIR/frontend/src/assets/styles/global.css" << 'EOF'
@import 'tailwindcss/base';
@import 'tailwindcss/components';
@import 'tailwindcss/utilities';

/* Custom styles */
body {
  background-color: #f7f9fc;
  color: #333;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Helvetica Neue', sans-serif;
}

.gradient-bg {
  background: linear-gradient(to right, #0066ff, #9900ff);
}

.spinner {
  border: 3px solid rgba(0, 0, 0, 0.1);
  border-top: 3px solid #0066ff;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
EOF

cat > "$BASE_DIR/frontend/src/contexts/WalletContext.tsx" << 'EOF'
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
EOF

cat > "$BASE_DIR/frontend/src/contexts/UserContext.tsx" << 'EOF'
import React, { createContext, useState, useEffect, useContext, ReactNode } from 'react';
import { useWallet } from './WalletContext';
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
EOF

cat > "$BASE_DIR/frontend/src/services/api.ts" << 'EOF'
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
EOF

cat > "$BASE_DIR/frontend/src/components/common/Header.tsx" << 'EOF'
import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useWallet } from '../../contexts/WalletContext';
import { useUser } from '../../contexts/UserContext';

const Header: React.FC = () => {
  const { connected, address, balance, connect, disconnect } = useWallet();
  const { score } = useUser();
  const location = useLocation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [walletMenuOpen, setWalletMenuOpen] = useState(false);

  // Format wallet address for display
  const formatAddress = (addr: string): string => {
    if (!addr) return '';
    return `${addr.substring(0, 6)}...${addr.substring(addr.length - 4)}`;
  };
  
  // Format balance for display
  const formatBalance = (bal: string): string => {
    if (!bal) return '0';
    // Convert wei to EGLD (1 EGLD = 10^18 wei)
    const egld = parseInt(bal) / 1000000000000000000;
    return egld.toFixed(4);
  };

  // Navigation links
  const navLinks = [
    { name: 'Home', path: '/' },
    { name: 'Dashboard', path: '/dashboard' },
    { name: 'Loans', path: '/loans' },
    { name: 'Pools', path: '/pools' },
    { name: 'Marketplace', path: '/marketplace' },
  ];

  // Check if a link is active
  const isActive = (path: string): boolean => {
    if (path === '/' && location.pathname === '/') return true;
    return path !== '/' && location.pathname.startsWith(path);
  };

  return (
    <header className="bg-white shadow-sm">
      <div className="container mx-auto px-4 py-3">
        <div className="flex justify-between items-center">
          {/* Logo */}
          <Link to="/" className="flex items-center">
            <span className="text-xl font-bold text-blue-600">social-fi</span>
            <span className="text-xl font-bold text-purple-600">Credit</span>
          </Link>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex space-x-6">
            {navLinks.map((link) => (
              <Link
                key={link.path}
                to={link.path}
                className={`font-medium ${
                  isActive(link.path)
                    ? 'text-blue-600 border-b-2 border-blue-600'
                    : 'text-gray-600 hover:text-blue-600'
                }`}
              >
                {link.name}
              </Link>
            ))}
          </nav>

          {/* Wallet Section */}
          <div className="hidden md:flex items-center space-x-4">
            {connected ? (
              <div className="relative">
                <button
                  onClick={() => setWalletMenuOpen(!walletMenuOpen)}
                  className="flex items-center bg-gray-100 hover:bg-gray-200 rounded-lg px-3 py-2 text-sm font-medium text-gray-800"
                >
                  {score && (
                    <span className="mr-2 flex items-center">
                      <span className="inline-flex items-center justify-center bg-blue-100 text-blue-800 text-xs font-medium rounded-full h-5 w-5 mr-1">
                        {score.current}
                      </span>
                    </span>
                  )}
                  <span>{formatAddress(address || '')}</span>
                  <span className="ml-2 text-green-500 font-semibold">{formatBalance(balance || '0')} EGLD</span>
                  <svg className="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7"></path>
                  </svg>
                </button>

                {walletMenuOpen && (
                  <div className="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-10">
                    <Link
                      to="/profile"
                      className="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                      onClick={() => setWalletMenuOpen(false)}
                    >
                      My Profile
                    </Link>
                    <button
                      onClick={() => {
                        disconnect();
                        setWalletMenuOpen(false);
                      }}
                      className="block w-full text-left px-4 py-2 text-sm text-red-700 hover:bg-gray-100"
                    >
                      Disconnect
                    </button>
                  </div>
                )}
              </div>
            ) : (
              <div className="flex space-x-2">
                <button
                  onClick={() => connect('extension')}
                  className="bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium"
                >
                  Connect Wallet
                </button>
              </div>
            )}
          </div>

          {/* Mobile menu button */}
          <div className="md:hidden flex items-center">
            {connected && (
              <button
                onClick={() => setWalletMenuOpen(!walletMenuOpen)}
                className="flex items-center bg-gray-100 hover:bg-gray-200 rounded-lg px-3 py-2 text-sm font-medium text-gray-800 mr-2"
              >
                <span>{formatAddress(address || '')}</span>
              </button>
            )}
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="text-gray-500 hover:text-gray-600"
            >
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                {mobileMenuOpen ? (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                ) : (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                )}
              </svg>
            </button>
          </div>
        </div>

        {/* Mobile menu */}
        {mobileMenuOpen && (
          <div className="mt-4 md:hidden">
            <div className="flex flex-col space-y-2">
              {navLinks.map((link) => (
                <Link
                  key={link.path}
                  to={link.path}
                  className={`px-3 py-2 rounded-md text-base font-medium ${
                    isActive(link.path)
                      ? 'bg-blue-100 text-blue-700'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-blue-600'
                  }`}
                  onClick={() => setMobileMenuOpen(false)}
                >
                  {link.name}
                </Link>
              ))}
              {!connected && (
                <button
                  onClick={() => {
                    connect('extension');
                    setMobileMenuOpen(false);
                  }}
                  className="mt-2 w-full bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium"
                >
                  Connect Wallet
                </button>
              )}
            </div>
          </div>
        )}
        
        {/* Mobile wallet menu */}
        {walletMenuOpen && mobileMenuOpen && (
          <div className="mt-2 border-t pt-2">
            <div className="flex justify-between items-center mb-2">
              <span className="text-sm text-gray-500">Balance:</span>
              <span className="text-green-500 font-semibold">{formatBalance(balance || '0')} EGLD</span>
            </div>
            {score && (
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-gray-500">Score:</span>
                <span className="flex items-center">
                  <span className="inline-flex items-center justify-center bg-blue-100 text-blue-800 text-sm font-medium rounded-full h-6 w-6">
                    {score.current}
                  </span>
                </span>
              </div>
            )}
            <div className="flex flex-col space-y-2 mt-2">
              <Link
                to="/profile"
                className="block px-3 py-2 rounded-md text-base font-medium text-gray-700 hover:bg-gray-50 hover:text-blue-600"
                onClick={() => {
                  setWalletMenuOpen(false);
                  setMobileMenuOpen(false);
                }}
              >
                My Profile
              </Link>
              <button
                onClick={() => {
                  disconnect();
                  setWalletMenuOpen(false);
                  setMobileMenuOpen(false);
                }}
                className="block text-left px-3 py-2 rounded-md text-base font-medium text-red-700 hover:bg-gray-50"
              >
                Disconnect
              </button>
            </div>
          </div>
        )}
      </div>
    </header>
  );
};

export default Header;
EOF

cat > "$BASE_DIR/frontend/src/components/common/Footer.tsx" << 'EOF'
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
EOF

cat > "$BASE_DIR/frontend/src/components/dashboard/ScoreDisplay.tsx" << 'EOF'
import React from 'react';
import { useUser } from '../../contexts/UserContext';

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
EOF

cat > "$BASE_DIR/frontend/src/pages/Home.tsx" << 'EOF'
import React from 'react';
import { Link } from 'react-router-dom';
import { useWallet } from '../contexts/WalletContext';

const Home: React.FC = () => {
  const { connected, connect } = useWallet();

  return (
    <div className="flex flex-col items-center">
      {/* Hero Section */}
      <section className="w-full bg-gradient-to-r from-blue-500 to-purple-600 text-white py-16 px-4 rounded-lg">
        <div className="max-w-4xl mx-auto text-center">
          <h1 className="text-4xl md:text-5xl font-bold mb-6">
            Your Social Reputation Is Now Your Credit Score
          </h1>
          <p className="text-xl md:text-2xl mb-8 opacity-90">
            social-fi Credit revolutionizes DeFi with zero-collateral loans based on your social reputation.
          </p>
          
          {!connected ? (
            <div>
              <button 
                onClick={() => connect('extension')}
                className="bg-white text-blue-600 hover:bg-blue-50 font-medium py-3 px-8 rounded-lg text-lg shadow-lg mr-4"
              >
                Connect Wallet
              </button>
              <Link 
                to="/loans" 
                className="inline-block mt-4 md:mt-0 bg-transparent border-2 border-white hover:bg-white hover:text-blue-600 text-white font-medium py-3 px-8 rounded-lg text-lg transition-colors"
              >
                Learn More
              </Link>
            </div>
          ) : (
            <div>
              <Link 
                to="/dashboard" 
                className="bg-white text-blue-600 hover:bg-blue-50 font-medium py-3 px-8 rounded-lg text-lg shadow-lg mr-4"
              >
                Go to Dashboard
              </Link>
              <Link 
                to="/loans" 
                className="inline-block mt-4 md:mt-0 bg-transparent border-2 border-white hover:bg-white hover:text-blue-600 text-white font-medium py-3 px-8 rounded-lg text-lg transition-colors"
              >
                Get a Loan
              </Link>
            </div>
          )}
        </div>
      </section>

      {/* How It Works Section */}
      <section className="w-full py-16 px-4">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">How It Works</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {/* Step 1 */}
            <div className="bg-white rounded-lg shadow-md p-6 text-center">
              <div className="w-16 h-16 bg-blue-100 text-blue-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">1</div>
              <h3 className="text-xl font-semibold mb-3">Connect Your Social</h3>
              <p className="text-gray-600">Link your Twitter account to start building your reputation through the #ElizaOS hashtag.</p>
            </div>
            
            {/* Step 2 */}
            <div className="bg-white rounded-lg shadow-md p-6 text-center">
              <div className="w-16 h-16 bg-purple-100 text-purple-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">2</div>
              <h3 className="text-xl font-semibold mb-3">Build Reputation</h3>
              <p className="text-gray-600">Engage with the community by sharing resources, answering questions, and contributing positively.</p>
            </div>
            
            {/* Step 3 */}
            <div className="bg-white rounded-lg shadow-md p-6 text-center">
              <div className="w-16 h-16 bg-green-100 text-green-600 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold">3</div>
              <h3 className="text-xl font-semibold mb-3">Access Credit</h3>
              <p className="text-gray-600">Use your Community Score to access loans without traditional collateral requirements.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="w-full py-16 px-4 bg-gray-50">
        <div className="max-w-4xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">Key Features</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path fillRule="evenodd" d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Zero-Collateral Loans</h3>
                <p className="text-gray-600">Access crypto loans without locking up your assets as collateral.</p>
              </div>
            </div>
            
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path d="M9 6a3 3 0 11-6 0 3 3 0 016 0zM17 6a3 3 0 11-6 0 3 3 0 016 0zM12.93 17c.046-.327.07-.66.07-1a6.97 6.97 0 00-1.5-4.33A5 5 0 0119 16v1h-6.07zM6 11a5 5 0 015 5v1H1v-1a5 5 0 015-5z"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Community-Based Trust</h3>
                <p className="text-gray-600">Your reputation within the community determines your creditworthiness.</p>
              </div>
            </div>
            
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path fillRule="evenodd" d="M12 7a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0V8.414l-4.293 4.293a1 1 0 01-1.414 0L8 10.414l-4.293 4.293a1 1 0 01-1.414-1.414l5-5a1 1 0 011.414 0L11 10.586 14.586 7H12z" clipRule="evenodd"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Dynamic Scoring</h3>
                <p className="text-gray-600">Real-time updates to your Community Score based on social interactions.</p>
              </div>
            </div>
            
            <div className="flex">
              <div className="mr-4 mt-1 text-blue-500">
                <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                  <path d="M3 12v3c0 1.657 3.134 3 7 3s7-1.343 7-3v-3c0 1.657-3.134 3-7 3s-7-1.343-7-3z"></path>
                  <path d="M3 7v3c0 1.657 3.134 3 7 3s7-1.343 7-3V7c0 1.657-3.134 3-7 3S3 8.657 3 7z"></path>
                  <path d="M17 5c0 1.657-3.134 3-7 3S3 6.657 3 5s3.134-3 7-3 7 1.343 7 3z"></path>
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-semibold mb-2">Risk-Based Pools</h3>
                <p className="text-gray-600">Different liquidity pools based on risk tolerance with varying interest rates.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="w-full py-16 px-4">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl font-bold mb-6">Ready to Get Started?</h2>
          <p className="text-xl text-gray-600 mb-8">
            Join the revolution in decentralized finance and start building your credit today.
          </p>
          
          {!connected ? (
            <button 
              onClick={() => connect('extension')}
              className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-8 rounded-lg text-lg shadow-lg"
            >
              Connect Wallet
            </button>
          ) : (
            <Link 
              to="/dashboard" 
              className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-8 rounded-lg text-lg shadow-lg"
            >
              View Dashboard
            </Link>
          )}
        </div>
      </section>
    </div>
  );
};

export default Home;
EOF

cat > "$BASE_DIR/frontend/src/pages/Dashboard.tsx" << 'EOF'
import React from 'react';
import { useWallet } from '../contexts/WalletContext';
import ScoreDisplay from '../components/dashboard/ScoreDisplay';
import { Link } from 'react-router-dom';

const Dashboard: React.FC = () => {
  const { connected, address } = useWallet();

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center py-12">
        <div className="text-center">
          <h1 className="text-3xl font-bold mb-4">Connect Wallet to Access Dashboard</h1>
          <p className="text-gray-600 mb-8">
            Please connect your MultiversX wallet to view your dashboard and access all features.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div>
      <h1 className="text-3xl font-bold mb-6">Dashboard</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Community Score */}
        <div className="col-span-1">
          <ScoreDisplay />
        </div>
        
        {/* Active Loans */}
        <div className="col-span-1 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">Your Active Loans</h2>
          
          <div className="space-y-4">
            {/* Placeholder when no loans */}
            <div className="text-center py-6">
              <p className="text-gray-500 mb-4">You don't have any active loans</p>
              <Link 
                to="/loans" 
                className="inline-block bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 text-sm font-medium"
              >
                Get a Loan
              </Link>
            </div>
            
            {/* Loan items would be rendered here in a real implementation */}
          </div>
        </div>
        
        {/* Recent Activity */}
        <div className="col-span-1 bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4">Recent Activity</h2>
          
          <div className="space-y-4">
            {/* Activity items */}
            <div className="border-l-4 border-green-500 pl-4 py-1">
              <p className="text-sm font-medium">Score Increased</p>
              <p className="text-xs text-gray-500">Your community score increased by 5 points</p>
              <p className="text-xs text-gray-400">2 hours ago</p>
            </div>
            
            <div className="border-l-4 border-blue-500 pl-4 py-1">
              <p className="text-sm font-medium">Twitter Connected</p>
              <p className="text-xs text-gray-500">Successfully connected your Twitter account</p>
              <p className="text-xs text-gray-400">1 day ago</p>
            </div>
            
            <div className="border-l-4 border-purple-500 pl-4 py-1">
              <p className="text-sm font-medium">Wallet Connected</p>
              <p className="text-xs text-gray-500">Successfully connected your MultiversX wallet</p>
              <p className="text-xs text-gray-400">1 day ago</p>
            </div>
          </div>
        </div>
      </div>
      
      {/* Twitter Feed */}
      <div className="mt-8 bg-white rounded-lg shadow-md p-6">
        <h2 className="text-xl font-semibold mb-4">Community Activity with #ElizaOS</h2>
        
        <div className="space-y-4">
          {/* Placeholder for Twitter feed */}
          <p className="text-gray-500 text-center py-8">
            Connect your Twitter account to view community activity
          </p>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
EOF

cat > "$BASE_DIR/frontend/src/pages/Loans.tsx" << 'EOF'
import React, { useState, useEffect } from 'react';
import { useWallet } from '../contexts/WalletContext';
import { useUser } from '../contexts/UserContext';
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
EOF

# Adicionar scripts para configuração de ambiente e deployment
cat > "$BASE_DIR/scripts/setup_environment.sh" << 'EOF'
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
EOF

cat > "$BASE_DIR/scripts/deploy_contracts.sh" << 'EOF'
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
EOF

cat > "$BASE_DIR/scripts/run_tests.sh" << 'EOF'
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
EOF

# Tornar scripts executáveis
chmod +x "$BASE_DIR/scripts/setup_environment.sh"
chmod +x "$BASE_DIR/scripts/deploy_contracts.sh"
chmod +x "$BASE_DIR/scripts/run_tests.sh"

# Adicionar arquivos de CI/CD
cat > "$BASE_DIR/.github/workflows/test.yml" << 'EOF'
name: Run Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test-contracts:
    name: Test Smart Contracts
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Test Contracts
        run: |
          cd smart-contracts
          cargo test

  test-backend:
    name: Test Backend
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.9'
      - name: Install dependencies
        run: |
          cd backend
          python -m pip install --upgrade pip
          pip install -r requirements.txt
      - name: Run tests
        run: |
          cd backend
          python -m pytest

  test-frontend:
    name: Test Frontend
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      - name: Install dependencies
        run: |
          cd frontend
          npm install
      - name: Run tests
        run: |
          cd frontend
          npm test
EOF

cat > "$BASE_DIR/.github/workflows/deploy.yml" << 'EOF'
name: Deploy

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-deploy:
    name: Build and Deploy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      # Build Smart Contracts
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Build Smart Contracts
        run: |
          cd smart-contracts
          cargo build --release
      
      # Build Frontend
      - name: Set up Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '16'
      - name: Build Frontend
        run: |
          cd frontend
          npm install
          npm run build
          
      # Build and push Docker images
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2
      - name: Login to DockerHub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}
          
      - name: Build and push Backend
        uses: docker/build-push-action@v3
        with:
          context: ./backend
          push: true
          tags: social-ficredit/backend:latest,social-ficredit/backend:${{ github.ref_name }}
          
      - name: Build and push ElizaOS
        uses: docker/build-push-action@v3
        with:
          context: ./backend
          file: ./backend/Dockerfile.eliza
          push: true
          tags: social-ficredit/eliza:latest,social-ficredit/eliza:${{ github.ref_name }}
          
      - name: Build and push Frontend
        uses: docker/build-push-action@v3
        with:
          context: ./frontend
          push: true
          tags: social-ficredit/frontend:latest,social-ficredit/frontend:${{ github.ref_name }}
          
      # Deploy to server
      - name: Deploy to server
        uses: appleboy/ssh-action@master
        with:
          host: ${{ secrets.SSH_HOST }}
          username: ${{ secrets.SSH_USERNAME }}
          key: ${{ secrets.SSH_PRIVATE_KEY }}
          script: |
            cd /opt/social-fi-credit
            git pull
            docker-compose pull
            docker-compose up -d
            docker system prune -af
EOF

# Criar arquivos de documentação
cat > "$BASE_DIR/docs/technical.md" << 'EOF'
# social-fi Credit - Technical Documentation

## Architecture Overview

social-fi Credit is a decentralized finance platform that enables zero-collateral loans based on social reputation. The platform consists of several key components:

### 1. Smart Contracts (MultiversX/Rust)

The blockchain layer consists of four main smart contracts:

- **ReputationScore**: Manages user reputation scores derived from social interactions
- **LoanController**: Handles loan requests, approvals, and repayments
- **LiquidityPool**: Manages funds provided by investors with different risk levels
- **DebtToken**: Implements NFTs representing tokenized debt

### 2. ElizaOS (Python)

ElizaOS is the social oracle system that:

- Monitors Twitter for interactions with the #ElizaOS hashtag
- Analyzes sentiment and content of social media posts
- Calculates Community Scores based on social interactions
- Updates the blockchain with reputation scores

### 3. Backend API (Python/FastAPI)

The backend provides a RESTful API that:

- Communicates with the blockchain
- Processes user requests
- Manages Twitter authentication
- Provides data for the frontend application

### 4. Frontend (React/TypeScript)

The user interface allows users to:

- Connect their MultiversX wallet
- Link their Twitter account
- View their Community Score
- Request zero-collateral loans
- Repay loans and track their credit history

## Data Flow

1. User connects Twitter account and interacts with the community using #ElizaOS
2. ElizaOS monitors and analyzes these interactions
3. Community Score is calculated and updated on the blockchain
4. User can request loans based on their score
5. LoanController checks eligibility and approves loans
6. Funds are transferred from liquidity pools to the user
7. User repays loans with interest by the due date
8. Timely repayments increase user's score, defaults decrease it

## Technical Specifications

### Smart Contracts

- **Language**: Rust
- **Blockchain**: MultiversX
- **Standards**: ESDT (MultiversX Standard Digital Token)

### Backend

- **Language**: Python 3.9+
- **Framework**: FastAPI
- **Database**: None (blockchain as source of truth)
- **APIs**: Twitter API for social monitoring

### Frontend

- **Framework**: React 18
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **State Management**: React Context API
- **Wallet Connection**: MultiversX SDK

## Security Considerations

1. **Score Manipulation Prevention**:
   - Oracle is the only entity allowed to update reputation scores
   - Multiple data points are considered for score calculation
   - Anti-spam measures detect artificial engagement

2. **Risk Management**:
   - Loan amounts are capped based on Community Score
   - Progressive loan increase with successful repayments
   - Multiple risk pools for investors

3. **Privacy**:
   - Only public social media data is analyzed
   - Users explicitly opt-in to connect their accounts
   - Personal data is not stored centrally

## Deployment

The application is deployed using Docker containers:

- **Backend API**: FastAPI application
- **ElizaOS**: Python service for social monitoring
- **Frontend**: React app served via Nginx

The deployment process is automated via GitHub Actions CI/CD pipeline.
EOF

cat > "$BASE_DIR/docs/user-guide.md" << 'EOF'
# social-fi Credit - User Guide

Welcome to social-fi Credit, the revolutionary DeFi platform that provides zero-collateral loans based on your social reputation. This guide will help you get started and make the most of the platform.

## Getting Started

### 1. Connect Your Wallet

Before you can use social-fi Credit, you need to connect your MultiversX wallet:

1. Click the "Connect Wallet" button in the top-right corner of the screen
2. Choose your preferred connection method:
   - MultiversX Web Wallet
   - MultiversX DeFi Wallet
   - Ledger hardware wallet
3. Approve the connection request in your wallet

### 2. Connect Your Twitter Account

To start building your Community Score:

1. Go to your Profile page
2. Click "Connect Twitter Account"
3. Authorize social-fi Credit to access your Twitter data
4. Your Twitter account is now linked

### 3. Build Your Community Score

Your Community Score is the key to accessing loans without collateral. Here's how to build it:

- **Engage with the community**: Use the #ElizaOS hashtag in your tweets
- **Share valuable content**: Post helpful resources related to crypto and DeFi
- **Answer questions**: Help other community members with their questions
- **Maintain positive interactions**: The sentiment of your posts affects your score

The more positive engagement you create, the higher your score will climb!

## Using the Platform

### Checking Your Score

1. Go to the Dashboard to see your current Community Score
2. The score ranges from 0 to 1000
3. You'll see if you're eligible for loans and your maximum loan amount
4. Check the "Recent Activity" section to see what affected your score

### Requesting a Loan

Once your score is high enough (minimum 50 points), you can request a loan:

1. Go to the Loans page
2. Enter the amount you wish to borrow (within your limit)
3. Select the loan duration (7-90 days)
4. Review the interest rate and repayment amount
5. Click "Apply for Loan"
6. Sign the transaction in your wallet
7. Funds will be transferred to your wallet after approval

### Repaying a Loan

To repay your loan and maintain a good standing:

1. Go to the Loans page
2. Find your active loan in the list
3. Click the "Repay" button
4. Sign the transaction in your wallet

Timely repayments will increase your Community Score, allowing you to borrow larger amounts in the future.

### Investing in Pools

If you want to earn interest by providing liquidity:

1. Go to the Pools page
2. Choose a risk level that matches your preference:
   - AAA Pool: Lowest risk, lowest returns
   - BBB Pool: Medium risk, medium returns
   - CCC Pool: Highest risk, highest returns
3. Enter the amount you wish to invest
4. Sign the transaction in your wallet
5. You'll start earning interest based on the pool's performance

## Tips for Success

1. **Be consistent**: Regular positive interactions build your score faster
2. **Quality over quantity**: Meaningful contributions count more than simple likes
3. **Repay on time**: Late payments significantly impact your score
4. **Start small**: Begin with smaller loans to build your credit history
5. **Monitor your score**: Check your Dashboard regularly to see how your activities impact your score

## Troubleshooting

If you encounter any issues:

- **Wallet connection problems**: Try refreshing the page or reconnecting
- **Twitter connection issues**: Disconnect and reconnect your Twitter account
- **Transaction failures**: Check that you have enough EGLD for gas fees
- **Score not updating**: Score updates may take up to 24 hours to process

For additional help, join our Discord community or contact support at support@social-ficredit.io.
EOF

cat > "$BASE_DIR/docs/api-docs.md" << 'EOF'
# social-fi Credit API Documentation

This document provides details on the RESTful API endpoints available for the social-fi Credit platform.

## Base URL

All API endpoints are relative to the base URL:

```
https://api.social-ficredit.io
```

For development:

```
http://localhost:8000
```

## Authentication

All endpoints that require authentication need a valid MultiversX address signature. Include the following headers:

```
X-Address: erd1...
X-Signature: <signature>
```

The signature should be created by signing a challenge message with the MultiversX wallet.

## Endpoints

### User Endpoints

#### Get User Profile

```
GET /api/users/{address}
```

Returns user profile information including Community Score, loans taken, and Twitter connection status.

**Response:**

```json
{
  "address": "erd1...",
  "twitterId": "1234567890",
  "twitterHandle": "user123",
  "score": 750,
  "loansTaken": 2,
  "loansRepaid": 1,
  "registeredAt": "2025-04-15T10:30:00Z"
}
```

#### Get User Score

```
GET /api/users/{address}/score
```

Returns detailed information about a user's Community Score.

**Response:**

```json
{
  "current": 750,
  "max": 1000,
  "eligibleForLoan": true,
  "maxLoanAmount": "1.50"
}
```

#### Connect Twitter Account

```
POST /api/users/{address}/connect-twitter
```

Links a Twitter account to the user's profile.

**Request:**

```json
{
  "twitterHandle": "user123",
  "oauthToken": "oauth_token_here"
}
```

**Response:**

```json
{
  "status": "success",
  "message": "Twitter account connected successfully"
}
```

#### Get Twitter Stats

```
GET /api/users/{address}/twitter-stats
```

Returns statistics about the user's Twitter activity related to the platform.

**Response:**

```json
{
  "positive_mentions": 12,
  "technical_answers": 3,
  "resources_shared": 5,
  "total_likes_received": 87,
  "total_retweets": 15,
  "last_updated": "2025-04-28T15:20:30Z"
}
```

### Loan Endpoints

#### Get Loans

```
GET /api/loans
```

Returns a list of loans. Can be filtered by address and status.

**Query Parameters:**

- `address`: Filter by borrower address
- `status`: Filter by loan status (Active, Repaid, Defaulted)
- `skip`: Number of items to skip (pagination)
- `limit`: Maximum number of items to return (pagination)

**Response:**

```json
[
  {
    "id": "1",
    "borrower": "erd1...",
    "amount": "0.5",
    "repayment_amount": "0.525",
    "interest_rate": 5.0,
    "created_at": "2025-04-10T12:00:00Z",
    "due_date": "2025-05-10T12:00:00Z",
    "status": "Active",
    "nft_id": null
  },
  {
    "id": "2",
    "borrower": "erd1...",
    "amount": "1.0",
    "repayment_amount": "1.08",
    "interest_rate": 8.0,
    "created_at": "2025-03-15T10:30:00Z",
    "due_date": "2025-04-15T10:30:00Z",
    "status": "Repaid",
    "nft_id": "DEBTFT-abcdef"
  }
]
```

#### Get Loan by ID

```
GET /api/loans/{loan_id}
```

Returns detailed information about a specific loan.

**Response:**

Same as a single loan item from the list endpoint.

#### Request Loan

```
POST /api/loans/request
```

Creates a loan request that needs to be signed by the user.

**Request:**

```json
{
  "amount": "0.5",
  "duration_days": 30,
  "token_id": "EGLD"
}
```

**Response:**

```json
{
  "status": "success",
  "message": "Loan request created. Please sign the transaction in your wallet.",
  "transaction": {
    "nonce": 42,
    "value": "0",
    "receiver": "erd1...",
    "sender": "erd1...",
    "gasPrice": 1000000000,
    "gasLimit": 500000,
    "data": "requestLoan@0500@1e",
    "chainID": "D",
    "version": 1
  }
}
```

#### Repay Loan

```
POST /api/loans/repay
```

Creates a transaction to repay an existing loan.

**Request:**

```json
{
  "loan_id": "1",
  "token_id": "EGLD"
}
```

**Response:**

```json
{
  "status": "success",
  "message": "Loan repayment prepared. Please sign the transaction in your wallet.",
  "transaction": {
    "nonce": 43,
    "value": "0.525",
    "receiver": "erd1...",
    "sender": "erd1...",
    "gasPrice": 1000000000,
    "gasLimit": 500000,
    "data": "repayLoan@01",
    "chainID": "D",
    "version": 1
  }
}
```

#### Calculate Interest

```
GET /api/loans/calculate-interest?amount=0.5&address=erd1...
```

Calculates the interest rate and repayment amount for a loan.

**Query Parameters:**

- `amount`: Loan amount
- `address`: Borrower address

**Response:**

```json
{
  "interestRate": 5.0,
  "repaymentAmount": "0.525",
  "totalInterest": "0.025"
}
```

### Pool Endpoints

#### Get Pools

```
GET /api/pools
```

Returns a list of liquidity pools.

**Response:**

```json
[
  {
    "id": "AAA",
    "name": "AAA Pool",
    "risk_level": "Low",
    "total_liquidity": "100.5",
    "available_liquidity": "50.2",
    "current_apy": 3.5,
    "min_score_required": 750
  },
  {
    "id": "BBB",
    "name": "BBB Pool",
    "risk_level": "Medium",
    "total_liquidity": "75.3",
    "available_liquidity": "25.8",
    "current_apy": 7.2,
    "min_score_required": 500
  },
  {
    "id": "CCC",
    "name": "CCC Pool",
    "risk_level": "High",
    "total_liquidity": "45.1",
    "available_liquidity": "10.6",
    "current_apy": 12.5,
    "min_score_required": 250
  }
]
```

#### Get Pool by ID

```
GET /api/pools/{pool_id}
```

Returns detailed information about a specific pool.

**Response:**

Same as a single pool item from the list endpoint, plus additional details about active loans and performance metrics.

#### Provide Liquidity

```
POST /api/pools/provide
```

Creates a transaction to provide liquidity to a pool.

**Request:**

```json
{
  "pool_id": "AAA",
  "amount": "10.0",
  "token_id": "EGLD"
}
```

**Response:**

```json
{
  "status": "success",
  "message": "Liquidity provision prepared. Please sign the transaction in your wallet.",
  "transaction": {
    "nonce": 44,
    "value": "10.0",
    "receiver": "erd1...",
    "sender": "erd1...",
    "gasPrice": 1000000000,
    "gasLimit": 500000,
    "data": "provideLiquidity@414141",
    "chainID": "D",
    "version": 1
  }
}
```

#### Withdraw Liquidity

```
POST /api/pools/withdraw
```

Creates a transaction to withdraw liquidity from a pool.

**Request:**

```json
{
  "pool_id": "AAA",
  "amount": "5.0"
}
```

**Response:**

```json
{
  "status": "success",
  "message": "Liquidity withdrawal prepared. Please sign the transaction in your wallet.",
  "transaction": {
    "nonce": 45,
    "value": "0",
    "receiver": "erd1...",
    "sender": "erd1...",
    "gasPrice": 1000000000,
    "gasLimit": 500000,
    "data": "withdrawLiquidity@414141@05000000",
    "chainID": "D",
    "version": 1
  }
}
```

## Error Handling

All endpoints return standard HTTP status codes:

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

Error responses include a JSON body with details:

```json
{
  "status": "error",
  "code": 400,
  "detail": "Invalid amount: must be greater than zero"
}
```

## Rate Limiting

API requests are limited to 100 requests per minute per IP address. Exceeding this limit will result in a `429 Too Many Requests` response.

## Versioning

The current API version is v1. All endpoints should be prefixed with `/api` (already included in the examples above).

Future versions will be accessible via `/api/v2/`, etc.
EOF

# Criar mais arquivos do frontend
cat > "$BASE_DIR/frontend/Dockerfile" << 'EOF'
# Build stage
FROM node:16-alpine as build

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy build files from build stage
COPY --from=build /app/build /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
EOF

cat > "$BASE_DIR/frontend/nginx.conf" << 'EOF'
server {
    listen 80;
    server_name localhost;

    root /usr/share/nginx/html;
    index index.html index.htm;

    # Enable gzip compression
    gzip on;
    gzip_disable "msie6";
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_buffers 16 8k;
    gzip_http_version 1.1;
    gzip_min_length 256;
    gzip_types
        application/atom+xml
        application/javascript
        application/json
        application/ld+json
        application/manifest+json
        application/rss+xml
        application/vnd.geo+json
        application/vnd.ms-fontobject
        application/x-font-ttf
        application/x-web-app-manifest+json
        application/xhtml+xml
        application/xml
        font/opentype
        image/bmp
        image/svg+xml
        image/x-icon
        text/cache-manifest
        text/css
        text/plain
        text/vcard
        text/x-component
        text/x-cross-domain-policy;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy
    location /api/ {
        proxy_pass http://backend:8000/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # Error handling
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
    }
}
EOF

cat > "$BASE_DIR/frontend/public/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%PUBLIC_URL%/favicon.ico" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="theme-color" content="#0066ff" />
    <meta
      name="description"
      content="social-fi Credit - Zero-collateral DeFi loans based on social reputation"
    />
    <link rel="apple-touch-icon" href="%PUBLIC_URL%/logo192.png" />
    <link rel="manifest" href="%PUBLIC_URL%/manifest.json" />
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <title>social-fi Credit | Social Reputation DeFi</title>
  </head>
  <body>
    <noscript>You need to enable JavaScript to run this app.</noscript>
    <div id="root"></div>
  </body>
</html>
EOF

cat > "$BASE_DIR/frontend/public/manifest.json" << 'EOF'
{
  "short_name": "social-fi Credit",
  "name": "social-fi Credit | Social Reputation DeFi",
  "icons": [
    {
      "src": "favicon.ico",
      "sizes": "64x64 32x32 24x24 16x16",
      "type": "image/x-icon"
    },
    {
      "src": "logo192.png",
      "type": "image/png",
      "sizes": "192x192"
    },
    {
      "src": "logo512.png",
      "type": "image/png",
      "sizes": "512x512"
    }
  ],
  "start_url": ".",
  "display": "standalone",
  "theme_color": "#0066ff",
  "background_color": "#ffffff"
}
EOF

# Finalizar a configuração
echo "echo 'Estrutura de projeto criada com sucesso!'"
echo "echo 'Para começar:'"
echo "echo '1. cd $BASE_DIR'"
echo "echo '2. ./scripts/setup_environment.sh'"
echo "echo '3. docker-compose up -d'"
EOF

# Tornar o script executável
chmod +x "$BASE_DIR/setup_project.sh"

# Criar arquivo .env com valores iniciais
cat > "$BASE_DIR/.env" << 'EOF'
# MultiversX Contract Addresses (serão atualizados durante o deploy)
REPUTATION_SCORE_ADDRESS=erd1_placeholder_address
LOAN_CONTROLLER_ADDRESS=erd1_placeholder_address
LIQUIDITY_POOL_ADDRESS=erd1_placeholder_address
DEBT_TOKEN_ADDRESS=erd1_placeholder_address

# Twitter API Credentials (ajuste com suas credenciais)
TWITTER_API_KEY=your_twitter_api_key
TWITTER_API_SECRET=your_twitter_api_secret
TWITTER_ACCESS_TOKEN=your_twitter_access_token
TWITTER_ACCESS_SECRET=your_twitter_access_secret

# Ambiente
ENVIRONMENT=development
DEBUG=true
API_HOST=0.0.0.0
API_PORT=8000
CORS_ORIGINS=["http://localhost:3000"]
EOF

# Mensagem final
echo "Script de configuração criado com sucesso!"
echo "Para executar: bash setup_project.sh"