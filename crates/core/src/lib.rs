//! GoldCoin Neural Core
//! 
//! The foundational layer of the world's first AI-native cryptocurrency wallet.
//! This module provides the core abstractions and traits that all other components build upon.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// The version of the Neural protocol
pub const NEURAL_PROTOCOL_VERSION: &str = "1.0.0";

/// Core wallet identity that evolves with usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralIdentity {
    /// Unique identifier for this wallet instance
    pub id: Uuid,
    /// Human-friendly name (can be changed)
    pub name: String,
    /// The wallet's personality matrix (learned over time)
    pub personality: PersonalityMatrix,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Number of transactions processed
    pub transaction_count: u64,
    /// Trust score (0.0 to 1.0)
    pub trust_score: f64,
}

/// Personality traits that the wallet develops
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityMatrix {
    /// How conservative vs aggressive in fee optimization
    pub risk_tolerance: f64,
    /// How verbose in explanations
    pub verbosity: f64,
    /// How proactive in suggestions
    pub proactivity: f64,
    /// Preference for automation
    pub automation_preference: f64,
    /// Security paranoia level
    pub security_strictness: f64,
}

impl Default for PersonalityMatrix {
    fn default() -> Self {
        Self {
            risk_tolerance: 0.5,
            verbosity: 0.5,
            proactivity: 0.5,
            automation_preference: 0.3,
            security_strictness: 0.7,
        }
    }
}

/// A neural transaction that can learn and optimize itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralTransaction {
    /// Transaction ID
    pub id: Uuid,
    /// Natural language intent
    pub intent: String,
    /// Parsed semantic meaning
    pub semantics: TransactionSemantics,
    /// Optimization parameters
    pub optimizations: OptimizationParams,
    /// Confidence score for this transaction
    pub confidence: f64,
    /// Learning feedback
    pub feedback: Option<TransactionFeedback>,
}

/// Semantic understanding of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSemantics {
    pub action: TransactionAction,
    pub recipient: Option<String>,
    pub amount: Option<f64>,
    pub urgency: Urgency,
    pub context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionAction {
    Send,
    Receive,
    Exchange,
    Stake,
    Query,
    Configure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Urgency {
    Immediate,
    Normal,
    Flexible,
    Scheduled(DateTime<Utc>),
}

/// Parameters for transaction optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationParams {
    pub optimize_for_fee: bool,
    pub optimize_for_speed: bool,
    pub optimize_for_privacy: bool,
    pub batch_with_others: bool,
    pub use_prediction: bool,
}

/// Feedback for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFeedback {
    pub success: bool,
    pub user_satisfaction: Option<f64>,
    pub actual_fee: Option<f64>,
    pub actual_time: Option<u64>,
    pub notes: Option<String>,
}

/// The core neural engine trait
#[async_trait]
pub trait NeuralEngine: Send + Sync {
    /// Process natural language input
    async fn process_input(&self, input: &str) -> Result<NeuralTransaction>;
    
    /// Learn from transaction feedback
    async fn learn(&mut self, transaction: &NeuralTransaction, feedback: TransactionFeedback) -> Result<()>;
    
    /// Predict optimal transaction parameters
    async fn predict_optimal(&self, semantics: &TransactionSemantics) -> Result<OptimizationParams>;
    
    /// Generate human-friendly explanation
    async fn explain(&self, transaction: &NeuralTransaction) -> Result<String>;
    
    /// Self-modify based on patterns
    async fn evolve(&mut self) -> Result<()>;
}

/// Memory system for the wallet
#[async_trait]
pub trait NeuralMemory: Send + Sync {
    /// Store a memory
    async fn remember(&mut self, key: &str, value: Vec<u8>) -> Result<()>;
    
    /// Recall a memory
    async fn recall(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Forget old or irrelevant memories
    async fn forget(&mut self, before: DateTime<Utc>) -> Result<u64>;
    
    /// Associate memories semantically
    async fn associate(&mut self, key1: &str, key2: &str, strength: f64) -> Result<()>;
    
    /// Find related memories
    async fn find_related(&self, key: &str, limit: usize) -> Result<Vec<String>>;
}

/// Security module that learns attack patterns
#[async_trait]
pub trait NeuralSecurity: Send + Sync {
    /// Analyze transaction for threats
    async fn analyze_threat(&self, transaction: &NeuralTransaction) -> Result<ThreatLevel>;
    
    /// Learn new attack pattern
    async fn learn_threat(&mut self, pattern: ThreatPattern) -> Result<()>;
    
    /// Generate security recommendation
    async fn recommend_security(&self, context: &SecurityContext) -> Result<Vec<SecurityAction>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    pub pattern_id: Uuid,
    pub description: String,
    pub indicators: Vec<String>,
    pub severity: ThreatLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub recent_transactions: Vec<NeuralTransaction>,
    pub network_state: NetworkState,
    pub user_behavior: UserBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    pub congestion: f64,
    pub known_attacks: Vec<String>,
    pub suspicious_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehavior {
    pub typical_amounts: Vec<f64>,
    pub typical_times: Vec<DateTime<Utc>>,
    pub typical_recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    RequireAdditionalAuth,
    DelayTransaction(u64),
    WarnUser(String),
    BlockTransaction,
    EnablePrivacyMode,
}

/// Event system for reactive behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeuralEvent {
    TransactionCreated(NeuralTransaction),
    TransactionCompleted(Uuid),
    ThreatDetected(ThreatLevel),
    LearningComplete(String),
    EvolutionTriggered,
    UserFeedback(TransactionFeedback),
}

/// The main Neural Wallet structure
pub struct NeuralWallet {
    pub identity: NeuralIdentity,
    pub engine: Arc<dyn NeuralEngine>,
    pub memory: Arc<dyn NeuralMemory>,
    pub security: Arc<dyn NeuralSecurity>,
}

impl NeuralWallet {
    /// Create a new Neural Wallet
    pub fn new(
        name: String,
        engine: Arc<dyn NeuralEngine>,
        memory: Arc<dyn NeuralMemory>,
        security: Arc<dyn NeuralSecurity>,
    ) -> Self {
        Self {
            identity: NeuralIdentity {
                id: Uuid::new_v4(),
                name,
                personality: PersonalityMatrix::default(),
                created_at: Utc::now(),
                transaction_count: 0,
                trust_score: 0.5,
            },
            engine,
            memory,
            security,
        }
    }
    
    /// Process a user command
    pub async fn process_command(&self, command: &str) -> Result<String> {
        // Parse the natural language command
        let transaction = self.engine.process_input(command).await?;
        
        // Check security
        let threat_level = self.security.analyze_threat(&transaction).await?;
        
        if matches!(threat_level, ThreatLevel::High | ThreatLevel::Critical) {
            return Ok("I've detected a potential security threat. Please verify this transaction.".to_string());
        }
        
        // Generate explanation
        let explanation = self.engine.explain(&transaction).await?;
        
        Ok(explanation)
    }
    
    /// Learn from user interaction
    pub async fn learn_from_interaction(&mut self, transaction: &NeuralTransaction, feedback: TransactionFeedback) -> Result<()> {
        // Update the engine's learning
        Arc::get_mut(&mut self.engine)
            .ok_or_else(|| anyhow::anyhow!("Cannot get mutable reference to engine"))?
            .learn(transaction, feedback.clone())
            .await?;
        
        // Store in memory
        let key = format!("tx:{}", transaction.id);
        let value = serde_json::to_vec(&(transaction, feedback.clone()))?;
        Arc::get_mut(&mut self.memory)
            .ok_or_else(|| anyhow::anyhow!("Cannot get mutable reference to memory"))?
            .remember(&key, value)
            .await?;
        
        // Update personality based on feedback
        self.adapt_personality(&feedback);
        
        Ok(())
    }
    
    /// Adapt personality based on user feedback
    fn adapt_personality(&mut self, feedback: &TransactionFeedback) {
        if let Some(satisfaction) = feedback.user_satisfaction {
            // Adjust personality traits based on satisfaction
            let adjustment = (satisfaction - 0.5) * 0.01; // Small incremental changes
            
            // If user is satisfied, reinforce current behavior
            // If not, adjust in opposite direction
            self.identity.personality.proactivity += adjustment;
            self.identity.personality.verbosity += adjustment;
            
            // Clamp values between 0 and 1
            self.identity.personality.proactivity = self.identity.personality.proactivity.clamp(0.0, 1.0);
            self.identity.personality.verbosity = self.identity.personality.verbosity.clamp(0.0, 1.0);
        }
        
        // Update trust score
        if feedback.success {
            self.identity.trust_score = (self.identity.trust_score * 0.99 + 0.01).min(1.0);
        } else {
            self.identity.trust_score = (self.identity.trust_score * 0.95).max(0.0);
        }
        
        self.identity.transaction_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_personality_matrix_default() {
        let personality = PersonalityMatrix::default();
        assert_eq!(personality.risk_tolerance, 0.5);
        assert_eq!(personality.security_strictness, 0.7);
    }
    
    #[test]
    fn test_neural_identity_creation() {
        let identity = NeuralIdentity {
            id: Uuid::new_v4(),
            name: "Test Wallet".to_string(),
            personality: PersonalityMatrix::default(),
            created_at: Utc::now(),
            transaction_count: 0,
            trust_score: 0.5,
        };
        
        assert_eq!(identity.name, "Test Wallet");
        assert_eq!(identity.transaction_count, 0);
    }
}