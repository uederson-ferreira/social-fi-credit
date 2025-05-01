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
