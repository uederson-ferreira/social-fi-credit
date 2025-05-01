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
