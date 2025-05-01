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
