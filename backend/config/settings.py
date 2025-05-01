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
