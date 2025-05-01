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
