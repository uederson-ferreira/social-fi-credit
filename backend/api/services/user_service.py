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
