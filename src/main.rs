//! GoldCoin Neural - The World's First AI-Native Cryptocurrency Wallet
//! 
//! This is the main entry point for the neural wallet.

use goldcoin_neural_core::{NeuralWallet, NeuralEngine, TransactionFeedback, NeuralSecurity, NeuralTransaction, ThreatLevel, ThreatPattern, SecurityContext, SecurityAction};
use goldcoin_neural_blockchain::{GoldcoinNeuralChain, GoldcoinNetwork, NeuralBlockchain};
use goldcoin_neural_ai::{GoldcoinAI, InMemoryNeuralMemory};
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, error, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("🧠⚡ GoldCoin Neural v0.1.0 - Initializing...");
    
    // ASCII art banner
    println!(r#"
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║   ██████╗  ██████╗ ██╗     ██████╗  ██████╗ ██████╗ ██╗███╗ ║
    ║  ██╔════╝ ██╔═══██╗██║     ██╔══██╗██╔════╝██╔═══██╗██║████╗║
    ║  ██║  ███╗██║   ██║██║     ██║  ██║██║     ██║   ██║██║██╔██║
    ║  ██║   ██║██║   ██║██║     ██║  ██║██║     ██║   ██║██║██║╚═╝║
    ║  ╚██████╔╝╚██████╔╝███████╗██████╔╝╚██████╗╚██████╔╝██║██║   ║
    ║   ╚═════╝  ╚═════╝ ╚══════╝╚═════╝  ╚═════╝ ╚═════╝ ╚═╝╚═╝   ║
    ║                                                               ║
    ║            N E U R A L   W A L L E T   v0.1.0                ║
    ║                                                               ║
    ║        "Your money just got a brain" 🧠                      ║
    ╚═══════════════════════════════════════════════════════════════╝
    "#);
    
    // Initialize components
    info!("Initializing neural components...");
    
    // Create memory system
    let memory = Arc::new(InMemoryNeuralMemory::new());
    
    // Initialize AI engine
    let ai_engine = Arc::new(GoldcoinAI::new(memory.clone()).await?);
    
    // Initialize blockchain interface
    let blockchain = Arc::new(GoldcoinNeuralChain::new(GoldcoinNetwork::Mainnet));
    
    // Create a mock security module for now
    let security = Arc::new(MockSecurity);
    
    // Create the neural wallet
    let mut wallet = NeuralWallet::new(
        "My Neural Wallet".to_string(),
        ai_engine.clone() as Arc<dyn NeuralEngine>,
        memory.clone(),
        security,
    );
    
    info!("✅ Neural wallet initialized successfully!");
    println!("\n🤖 Hello! I'm your AI financial companion.");
    println!("I understand natural language. Try commands like:");
    println!("  • 'Send 50 gold to Alice'");
    println!("  • 'What's my balance?'");
    println!("  • 'Optimize fees for next transaction'");
    println!("  • 'Show me transaction history'");
    println!("\nType 'help' for more commands or 'exit' to quit.\n");
    
    // Main interaction loop
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();
    
    loop {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        input.clear();
        reader.read_line(&mut input).await?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "exit" || input == "quit" {
            println!("👋 Goodbye! Your neural wallet is evolving even while you're away.");
            break;
        }
        
        if input == "help" {
            print_help();
            continue;
        }
        
        if input == "status" {
            print_wallet_status(&wallet).await;
            continue;
        }
        
        if input == "evolve" {
            println!("🧬 Triggering evolution cycle...");
            if let Some(engine) = Arc::get_mut(&mut ai_engine.clone()) {
                match engine.evolve().await {
                    Ok(_) => println!("✨ Evolution complete! I've adapted based on recent interactions."),
                    Err(e) => error!("Evolution failed: {}", e),
                }
            }
            continue;
        }
        
        // Process the command through the neural engine
        match wallet.process_command(input).await {
            Ok(response) => {
                println!("\n{}\n", response);
                
                // Simulate getting feedback
                if input.contains("send") || input.contains("pay") {
                    println!("📊 Was this transaction processed as expected? (yes/no/skip)");
                    
                    let mut feedback_line = String::new();
                    reader.read_line(&mut feedback_line).await?;
                    let feedback_input = feedback_line.trim().to_lowercase();
                    
                    if feedback_input == "yes" || feedback_input == "no" {
                        let satisfaction = if feedback_input == "yes" { 0.9 } else { 0.3 };
                        
                        // Create a mock transaction for feedback
                        let mock_tx = ai_engine.process_input(input).await?;
                        let feedback = TransactionFeedback {
                            success: feedback_input == "yes",
                            user_satisfaction: Some(satisfaction),
                            actual_fee: Some(1.5),
                            actual_time: Some(120),
                            notes: None,
                        };
                        
                        wallet.learn_from_interaction(&mock_tx, feedback).await?;
                        println!("📚 Thanks! I've learned from your feedback.");
                    }
                }
            }
            Err(e) => {
                error!("❌ Error processing command: {}", e);
                println!("I encountered an error: {}. Please try rephrasing.", e);
            }
        }
    }
    
    Ok(())
}

fn print_help() {
    println!(r#"
╔════════════════════════════════════════════════════════════════╗
║                    NEURAL WALLET COMMANDS                     ║
╠════════════════════════════════════════════════════════════════╣
║                                                                ║
║  TRANSACTIONS:                                                 ║
║    • Send [amount] [currency] to [recipient]                  ║
║    • Request [amount] from [sender]                           ║
║    • Exchange [amount] [from] to [to]                         ║
║                                                                ║
║  QUERIES:                                                      ║
║    • What's my balance?                                       ║
║    • Show transaction history                                 ║
║    • Check network status                                     ║
║                                                                ║
║  OPTIMIZATION:                                                 ║
║    • Optimize fees for next transaction                       ║
║    • Schedule transaction for lowest fees                     ║
║    • Enable privacy mode                                      ║
║                                                                ║
║  AI FEATURES:                                                  ║
║    • Learn my preferences                                     ║
║    • Predict best time to transact                           ║
║    • Analyze spending patterns                                ║
║                                                                ║
║  SYSTEM:                                                       ║
║    • status - Show wallet status                              ║
║    • evolve - Trigger AI evolution                            ║
║    • help - Show this help                                    ║
║    • exit - Quit the wallet                                   ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
    "#);
}

async fn print_wallet_status(wallet: &NeuralWallet) {
    println!(r#"
╔════════════════════════════════════════════════════════════════╗
║                      WALLET STATUS                            ║
╠════════════════════════════════════════════════════════════════╣"#);
    println!("║  Wallet ID: {:<50} ║", wallet.identity.id);
    println!("║  Name: {:<55} ║", wallet.identity.name);
    println!("║  Created: {:<52} ║", wallet.identity.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("║  Transactions: {:<51} ║", wallet.identity.transaction_count);
    println!("║  Trust Score: {:<52} ║", format!("{:.2}", wallet.identity.trust_score));
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                    PERSONALITY MATRIX                         ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  Risk Tolerance:    [{}] {:.0}%", 
        progress_bar(wallet.identity.personality.risk_tolerance),
        wallet.identity.personality.risk_tolerance * 100.0);
    println!("║  Verbosity:         [{}] {:.0}%",
        progress_bar(wallet.identity.personality.verbosity),
        wallet.identity.personality.verbosity * 100.0);
    println!("║  Proactivity:       [{}] {:.0}%",
        progress_bar(wallet.identity.personality.proactivity),
        wallet.identity.personality.proactivity * 100.0);
    println!("║  Automation:        [{}] {:.0}%",
        progress_bar(wallet.identity.personality.automation_preference),
        wallet.identity.personality.automation_preference * 100.0);
    println!("║  Security Level:    [{}] {:.0}%",
        progress_bar(wallet.identity.personality.security_strictness),
        wallet.identity.personality.security_strictness * 100.0);
    println!("╚════════════════════════════════════════════════════════════════╝");
}

fn progress_bar(value: f64) -> String {
    let filled = (value * 20.0) as usize;
    let empty = 20 - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

// Mock security implementation for demo
struct MockSecurity;

#[async_trait]
impl NeuralSecurity for MockSecurity {
    async fn analyze_threat(&self, _transaction: &NeuralTransaction) -> Result<ThreatLevel> {
        Ok(ThreatLevel::None)
    }
    
    async fn learn_threat(&mut self, _pattern: ThreatPattern) -> Result<()> {
        Ok(())
    }
    
    async fn recommend_security(&self, _context: &SecurityContext) -> Result<Vec<SecurityAction>> {
        Ok(vec![])
    }
}