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
