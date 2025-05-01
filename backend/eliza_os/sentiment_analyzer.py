import logging
from typing import Any
import re
import math
from sklearn.feature_extraction.text import CountVectorizer
from sklearn.naive_bayes import MultinomialNB
import spacy
import pytextrank

logger = logging.getLogger(__name__)

class SentimentAnalyzer:
    """
    Analyzes sentiment of text content related to social-fi Credit
    """
    
    def __init__(self):
        """Initialize the sentiment analyzer"""
        # Load SpaCy model
        try:
            self.nlp = spacy.load("en_core_web_sm")
            # Add TextRank component to pipeline
            self.nlp.add_pipe("textrank")
        except Exception as e:
            logger.error(f"Error loading NLP model: {e}")
            # Fallback to simple model
            self.nlp = None
            
        # Initialize classifier
        self.vectorizer = CountVectorizer(stop_words='english')
        self.classifier = MultinomialNB()
        
        # Train on sample data
        self._train_classifier()
        
    def _train_classifier(self):
        """Train the sentiment classifier on sample data"""
        # Sample training data
        texts = [
            "Great project, very innovative!", 
            "Loving the social-fi Credit system",
            "This is revolutionary for DeFi",
            "Amazing work with the zero-collateral loans",
            "Really impressed with ElizaOS integration",
            "The community score system is brilliant",
            "Excellent response to my question!",
            "Very helpful team",
            "This project is going to change everything",
            "Super excited about this",
            "This doesn't work at all",
            "Terrible experience with the loans",
            "Not impressed with the scoring system",
            "Too complicated to use",
            "Failed to get a loan despite high score",
            "Bug in the system, lost my money",
            "Poor documentation",
            "This is a scam",
            "Waste of time",
            "The team doesn't respond to questions"
        ]
        
        labels = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]  # 1 for positive, 0 for negative
        
        # Train the classifier
        try:
            X = self.vectorizer.fit_transform(texts)
            self.classifier.fit(X, labels)
            logger.info("Sentiment classifier trained successfully")
        except Exception as e:
            logger.error(f"Error training classifier: {e}")
        
    def analyze_text(self, text: str) -> float:
        """
        Analyze text sentiment and return a score from -1.0 to 1.0
        
        Args:
            text: Text to analyze
            
        Returns:
            Sentiment score (-1.0 to 1.0)
        """
        if not text:
            return 0.0
            
        # Clean text
        text = self._clean_text(text)
        
        # Get basic sentiment score from classifier
        basic_score = self._classify_sentiment(text)
        
        # Enhance with NLP analysis if available
        enhanced_score = basic_score
        if self.nlp:
            enhanced_score = self._enhance_sentiment(text, basic_score)
            
        return enhanced_score
    def _clean_text(self, text: str) -> str:
        """Clean and normalize text"""
        # Remove URLs
        text = re.sub(r'https?://\S+', '', text)
        
        # Remove user mentions
        text = re.sub(r'@\w+', '', text)
        
        # Remove hashtags but keep the text
        text = re.sub(r'#(\w+)', r'\1', text)
        
        # Remove extra whitespace
        text = re.sub(r'\s+', ' ', text).strip()
        
        return text
        
    def _classify_sentiment(self, text: str) -> float:
        """Use the trained classifier to get sentiment"""
        try:
            X = self.vectorizer.transform([text])
            # Get probability of positive class
            prob = self.classifier.predict_proba(X)[0][1]
            
            # Convert to -1.0 to 1.0 scale
            return (prob * 2) - 1.0
        except Exception as e:
            logger.error(f"Error classifying sentiment: {e}")
            return 0.0
            
    def _enhance_sentiment(self, text: str, basic_score: float) -> float:
        """Enhance sentiment analysis with NLP techniques"""
        try:
            doc = self.nlp(text)
            
            # Get keywords
            keywords = [phrase.text for phrase in doc._.phrases[:5]]
            
            # Check for sentiment modifiers
            intensifiers = ['very', 'super', 'really', 'extremely', 'absolutely']
            diminishers = ['somewhat', 'slightly', 'a bit', 'kind of', 'sort of']
            
            # Adjust score based on modifiers
            modifier = 1.0
            for intensifier in intensifiers:
                if intensifier in text.lower():
                    modifier *= 1.2
                    
            for diminisher in diminishers:
                if diminisher in text.lower():
                    modifier *= 0.8
                    
            # Ensure score stays in range
            enhanced_score = max(min(basic_score * modifier, 1.0), -1.0)
            
            return enhanced_score
            
        except Exception as e:
            logger.error(f"Error enhancing sentiment: {e}")
            return basic_score
