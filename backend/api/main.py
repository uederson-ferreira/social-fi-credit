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
