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
