#!/usr/bin/env python3
"""
GoldCoin Neural - Demo
A simplified demonstration of our AI-native cryptocurrency wallet capabilities.
"""

import json
import re
from typing import Optional, Dict, List
from dataclasses import dataclass
from datetime import datetime
import random

@dataclass
class TransactionIntent:
    """Represents parsed transaction intent from natural language"""
    action: str
    amount: Optional[float]
    recipient: Optional[str]
    urgency: str
    confidence: float

class ConversationalTransactionEngine:
    """
    Simplified natural language understanding for transaction intents.
    """
    
    def __init__(self):
        self.intent_patterns = {
            'send': ['send', 'pay', 'transfer', 'give', 'wire'],
            'receive': ['receive', 'request', 'invoice', 'bill'],
            'exchange': ['exchange', 'swap', 'convert', 'trade'],
            'stake': ['stake', 'delegate', 'lock'],
            'query': ['balance', 'status', 'check', 'show', 'what'],
        }
        
    def parse_intent(self, text: str) -> TransactionIntent:
        """Parse natural language into structured transaction intent"""
        text_lower = text.lower()
        
        # Determine action
        action = 'unknown'
        for intent, keywords in self.intent_patterns.items():
            if any(keyword in text_lower for keyword in keywords):
                action = intent
                break
        
        # Extract amount
        amount = self._extract_amount(text_lower)
        
        # Extract recipient
        recipient = self._extract_recipient(text_lower)
        
        # Determine urgency
        urgency = 'normal'
        if any(word in text_lower for word in ['now', 'immediately', 'urgent', 'asap']):
            urgency = 'immediate'
        elif any(word in text_lower for word in ['whenever', 'optimal', 'best time']):
            urgency = 'flexible'
        
        # Calculate confidence
        confidence = self._calculate_confidence(action, amount, recipient)
        
        return TransactionIntent(
            action=action,
            amount=amount,
            recipient=recipient,
            urgency=urgency,
            confidence=confidence
        )
    
    def _extract_amount(self, text: str) -> Optional[float]:
        """Extract numeric amount from text"""
        # Pattern for numbers possibly followed by currency
        pattern = r'(\d+\.?\d*)\s*(gold|gld|coins?|btc|usd)?'
        match = re.search(pattern, text)
        
        if match:
            try:
                return float(match.group(1))
            except ValueError:
                pass
        return None
    
    def _extract_recipient(self, text: str) -> Optional[str]:
        """Extract recipient from text"""
        # Look for patterns like "to [recipient]"
        if ' to ' in text:
            parts = text.split(' to ')
            if len(parts) > 1:
                recipient = parts[1].split()[0] if parts[1].split() else None
                return recipient
        return None
    
    def _calculate_confidence(self, action: str, amount: Optional[float], 
                             recipient: Optional[str]) -> float:
        """Calculate confidence score based on extracted information"""
        confidence = 0.3  # Base confidence
        
        if action != 'unknown':
            confidence += 0.3
        if amount is not None:
            confidence += 0.2
        if recipient is not None:
            confidence += 0.2
            
        return min(confidence, 1.0)

class PredictiveFeeOptimizer:
    """
    Mock fee prediction using simple heuristics.
    """
    
    def predict_fees(self, hours_ahead: int = 24) -> Dict[str, Dict]:
        """Predict fee rates for the next N hours"""
        current_hour = datetime.now().hour
        
        predictions = {}
        for h in range(hours_ahead):
            hour = (current_hour + h) % 24
            
            # Simulate congestion patterns (higher during business hours)
            if 9 <= hour <= 17:  # Business hours
                base_fee = 5.0
                congestion = 0.7
            elif 18 <= hour <= 22:  # Evening
                base_fee = 3.0
                congestion = 0.5
            else:  # Night/early morning
                base_fee = 1.0
                congestion = 0.2
            
            # Add some randomness
            noise = random.uniform(-0.5, 0.5)
            
            predictions[f"{h}h"] = {
                'low': max(0.5, base_fee * 0.5 + noise),
                'medium': max(1.0, base_fee + noise),
                'high': max(2.0, base_fee * 2 + noise),
                'congestion': congestion
            }
        
        return predictions
    
    def find_optimal_window(self, urgency: str = 'flexible') -> tuple:
        """Find the optimal time window for transaction"""
        predictions = self.predict_fees(24)
        
        if urgency == 'immediate':
            return 0, predictions['0h']['medium']
        
        # Find the hour with lowest fees
        best_hour = 0
        best_fee = float('inf')
        
        for hour_str, rates in predictions.items():
            hour = int(hour_str.replace('h', ''))
            if rates['low'] < best_fee:
                best_fee = rates['low']
                best_hour = hour
        
        return best_hour, best_fee

class NeuralWalletDemo:
    """
    Demo of the neural wallet capabilities
    """
    
    def __init__(self):
        self.engine = ConversationalTransactionEngine()
        self.fee_optimizer = PredictiveFeeOptimizer()
        self.transaction_history = []
        
    def process_command(self, command: str) -> str:
        """Process a command and return a response"""
        intent = self.engine.parse_intent(command)
        
        if intent.action == 'send':
            return self._handle_send(intent)
        elif intent.action == 'query':
            return self._handle_query(intent)
        elif intent.action == 'exchange':
            return self._handle_exchange(intent)
        else:
            return f"I understand you want to {intent.action}, but I need more specific instructions."
    
    def _handle_send(self, intent: TransactionIntent) -> str:
        """Handle send transaction"""
        response = "🚀 I understand you want to send "
        
        if intent.amount:
            response += f"{intent.amount} GLD "
        else:
            response += "some funds "
            
        if intent.recipient:
            response += f"to {intent.recipient}. "
        else:
            response += "to someone. "
        
        # Add fee optimization
        if intent.urgency == 'flexible':
            best_hour, best_fee = self.fee_optimizer.find_optimal_window('flexible')
            response += f"\n\n💡 For optimal fees ({best_fee:.2f} GLD), I recommend waiting {best_hour} hours. "
        elif intent.urgency == 'immediate':
            _, current_fee = self.fee_optimizer.find_optimal_window('immediate')
            response += f"\n\n⚡ For immediate processing, fee will be {current_fee:.2f} GLD. "
        
        response += f"\n\nConfidence: {intent.confidence:.0%}"
        
        # Record transaction
        self.transaction_history.append({
            'type': 'send',
            'amount': intent.amount,
            'recipient': intent.recipient,
            'timestamp': datetime.now().isoformat()
        })
        
        return response
    
    def _handle_query(self, intent: TransactionIntent) -> str:
        """Handle balance/status queries"""
        mock_balance = 1547.32  # Mock balance
        
        response = f"💰 Current Balance: {mock_balance} GLD\n"
        response += f"📊 Transactions Today: {len(self.transaction_history)}\n"
        response += f"🏦 Network Status: Healthy (30% congestion)\n"
        response += f"⛽ Current Fees: Low 1.2 GLD, Medium 3.1 GLD, High 8.7 GLD"
        
        return response
    
    def _handle_exchange(self, intent: TransactionIntent) -> str:
        """Handle exchange requests"""
        if intent.amount:
            # Mock exchange rate
            usd_rate = 127.45  # Mock GLD to USD rate
            usd_amount = intent.amount * usd_rate
            
            return f"💱 Exchange {intent.amount} GLD → ${usd_amount:.2f} USD\n" \
                   f"Rate: 1 GLD = ${usd_rate}\n" \
                   f"Fee: 0.5% ({intent.amount * 0.005:.3f} GLD)"
        else:
            return "💱 Current exchange rates:\n" \
                   "1 GLD = $127.45 USD\n" \
                   "1 GLD = €118.32 EUR\n" \
                   "1 GLD = ¥18,245 JPY"

def main():
    print("🧠⚡ GoldCoin Neural - AI Demo")
    print("=" * 50)
    
    wallet = NeuralWalletDemo()
    
    # Demo commands
    test_commands = [
        "Send 50 gold to Alice immediately",
        "What's my balance?",
        "Schedule a payment of 100 GLD when fees are lowest",
        "Exchange 200 gold to USD",
        "Send 25 GLD to Bob",
        "Check current exchange rates"
    ]
    
    for cmd in test_commands:
        print(f"\n🗣️  User: {cmd}")
        response = wallet.process_command(cmd)
        print(f"🤖 Neural: {response}")
        print("-" * 50)
    
    # Fee prediction demo
    print("\n📈 Fee Predictions for next 24 hours:")
    optimizer = PredictiveFeeOptimizer()
    predictions = optimizer.predict_fees(24)
    
    for hour in range(0, 24, 4):
        hour_key = f"{hour}h"
        if hour_key in predictions:
            pred = predictions[hour_key]
            print(f"  +{hour:2d}h: Low={pred['low']:.2f}, Med={pred['medium']:.2f}, High={pred['high']:.2f} GLD")
    
    best_hour, best_fee = optimizer.find_optimal_window('flexible')
    print(f"\n🎯 Optimal transaction window: in {best_hour} hours (fee: {best_fee:.2f} GLD)")
    
    print("\n✨ Demo complete! This shows the potential of AI-native cryptocurrency wallets.")

if __name__ == "__main__":
    main()