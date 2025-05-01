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
