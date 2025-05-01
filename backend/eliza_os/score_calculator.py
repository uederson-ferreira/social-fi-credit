from typing import Dict, List, Any
import logging
import math
from datetime import datetime

logger = logging.getLogger(__name__)

class ScoreCalculator:
    """
    Calculates user Community Score based on social media interactions
    """
    
    def __init__(self, sentiment_analyzer):
        """Initialize with sentiment analyzer"""
        self.sentiment_analyzer = sentiment_analyzer
        
        # Score multipliers for different types of interactions
        self.score_weights = {
            'positive_mention': 5,      # Mentioning project positively
            'technical_answer': 10,     # Answering technical questions
            'resource_sharing': 7,      # Sharing tutorials/resources
            'like_received': 1,         # Getting likes on project-related tweets
            'retweet_received': 3,      # Getting retweets on project-related tweets
            'follower_factor': 0.1      # Follower count factor (0.1 * log(followers))
        }
        
    def calculate_user_score(self, user_data: Dict[str, Any], tweets: List[Dict[str, Any]]) -> int:
        """
        Calculate a user's community score based on their activity
        
        Args:
            user_data: User profile data including follower counts
            tweets: List of tweet data from the user
            
        Returns:
            Calculated score as integer
        """
        if not user_data or not tweets:
            return 0
            
        total_score = 0
        
        # Base score from user metrics (followers)
        follower_count = user_data.get('followers_count', 0)
        follower_score = 0
        if follower_count > 0:
            import math
            follower_score = int(self.score_weights['follower_factor'] * math.log(follower_count + 1))
        
        total_score += follower_score
        
        # Analyze tweets
        for tweet in tweets:
            tweet_score = 0
            
            # Analyze sentiment
            sentiment = self.sentiment_analyzer.analyze_text(tweet['text'])
            
            # Score based on content type
            if self._is_positive_mention(tweet['text'], sentiment):
                tweet_score += self.score_weights['positive_mention']
                
            if self._is_technical_answer(tweet['text']):
                tweet_score += self.score_weights['technical_answer']
                
            if self._is_resource_sharing(tweet['text']):
                tweet_score += self.score_weights['resource_sharing']
                
            # Score based on engagement metrics
            tweet_score += tweet.get('like_count', 0) * self.score_weights['like_received']
            tweet_score += tweet.get('retweet_count', 0) * self.score_weights['retweet_received']
            
            # Apply time decay factor (more recent = more value)
            tweet_score = self._apply_time_decay(tweet_score, tweet['created_at'])
            
            total_score += tweet_score
            
        # Round to integer
        return int(total_score)
        
    def _is_positive_mention(self, text: str, sentiment: float) -> bool:
        """Detect if text is a positive project mention"""
        project_keywords = ['social-fi', 'credit', 'social-ficredit', 'elizaos']
        return any(keyword in text.lower() for keyword in project_keywords) and sentiment > 0.2
        
    def _is_technical_answer(self, text: str) -> bool:
        """Detect if text is answering a technical question"""
        technical_indicators = ['how to', 'problem', 'error', 'issue', 'fix', 'solution', 'code', 'contract']
        return any(indicator in text.lower() for indicator in technical_indicators) and len(text) > 100
        
    def _is_resource_sharing(self, text: str) -> bool:
        """Detect if text is sharing resources or tutorials"""
        resource_indicators = ['guide', 'tutorial', 'documentation', 'learn', 'http', 'https', 'github']
        return any(indicator in text.lower() for indicator in resource_indicators)
        
    def _apply_time_decay(self, score: float, created_at: datetime) -> float:
        """Apply time decay to score - more recent tweets worth more"""
        days_ago = (datetime.utcnow() - created_at).days
        if days_ago <= 1:
            return score  # Full value for tweets within last day
        elif days_ago <= 7:
            return score * 0.8  # 80% value for tweets within last week
        elif days_ago <= 30:
            return score * 0.5  # 50% value for tweets within last month
        else:
            return score * 0.2  # 20% value for older tweets
