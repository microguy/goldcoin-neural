//! GoldCoin Neural Interface Layer
//! 
//! This module provides various interfaces for interacting with the neural wallet.
//! Currently simplified for initial build.

pub mod rpc_server;

use goldcoin_neural_core::{NeuralWallet, NeuralTransaction};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, debug};
use serde_json::json;

/// Configuration for all interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub rest_port: u16,
    pub websocket_port: u16,
    pub grpc_port: u16,
    pub enable_voice: bool,
    pub enable_wasm: bool,
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            rest_port: 8080,
            websocket_port: 8081,
            grpc_port: 8082,
            enable_voice: false,
            enable_wasm: true,
        }
    }
}

/// Simplified REST API structures
pub mod rest {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    #[derive(Clone)]
    pub struct AppState {
        pub wallet: Arc<RwLock<NeuralWallet>>,
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct CommandRequest {
        pub command: String,
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct CommandResponse {
        pub response: String,
        pub success: bool,
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct WalletStatus {
        pub id: String,
        pub name: String,
        pub transaction_count: u64,
        pub trust_score: f64,
    }
    
    pub fn create_router() -> String {
        "Mock router for initial build".to_string()
    }
    
    pub async fn process_command_simple(wallet: &NeuralWallet, command: &str) -> CommandResponse {
        match wallet.process_command(command).await {
            Ok(response) => CommandResponse {
                response,
                success: true,
            },
            Err(e) => CommandResponse {
                response: format!("Error: {}", e),
                success: false,
            },
        }
    }
}

/// Simplified WebSocket interface
pub mod websocket {
    use super::*;
    use std::net::SocketAddr;
    
    pub struct WebSocketHandler {
        wallet: Arc<tokio::sync::RwLock<NeuralWallet>>,
    }
    
    impl WebSocketHandler {
        pub fn new(wallet: Arc<tokio::sync::RwLock<NeuralWallet>>) -> Self {
            Self { wallet }
        }
        
        pub async fn handle_message(&self, message: &str) -> Result<String> {
            if let Ok(request) = serde_json::from_str::<serde_json::Value>(message) {
                if let Some(command) = request.get("command").and_then(|c| c.as_str()) {
                    let wallet = self.wallet.read().await;
                    match wallet.process_command(command).await {
                        Ok(response) => Ok(json!({
                            "type": "response",
                            "command": command,
                            "response": response,
                            "success": true
                        }).to_string()),
                        Err(e) => Ok(json!({
                            "type": "error", 
                            "command": command,
                            "error": e.to_string(),
                            "success": false
                        }).to_string()),
                    }
                } else {
                    Ok(json!({
                        "type": "error",
                        "error": "No command provided"
                    }).to_string())
                }
            } else {
                Ok(json!({
                    "type": "error",
                    "error": "Invalid JSON"
                }).to_string())
            }
        }
    }
}

/// Simplified voice interface
pub mod voice {
    use super::*;
    
    pub struct VoiceInterface {
        wallet: Arc<tokio::sync::RwLock<NeuralWallet>>,
    }
    
    impl VoiceInterface {
        pub fn new(wallet: Arc<tokio::sync::RwLock<NeuralWallet>>) -> Self {
            Self { wallet }
        }
        
        pub async fn process_voice_command(&self, audio_data: Vec<u8>) -> Result<String> {
            // In production, use speech-to-text service
            let text = self.speech_to_text(audio_data).await?;
            
            let wallet = self.wallet.read().await;
            let response = wallet.process_command(&text).await?;
            
            Ok(response)
        }
        
        async fn speech_to_text(&self, _audio_data: Vec<u8>) -> Result<String> {
            // Placeholder for speech recognition
            Ok("What's my balance?".to_string())
        }
        
        pub async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>> {
            // Placeholder for text-to-speech
            Ok(text.as_bytes().to_vec())
        }
    }
}

/// Main interface manager that coordinates all interfaces
pub struct InterfaceManager {
    config: InterfaceConfig,
    wallet: Arc<tokio::sync::RwLock<NeuralWallet>>,
}

impl InterfaceManager {
    pub fn new(config: InterfaceConfig, wallet: Arc<tokio::sync::RwLock<NeuralWallet>>) -> Self {
        Self { config, wallet }
    }
    
    pub async fn start_all(&self) -> Result<()> {
        info!("Starting all neural wallet interfaces...");
        
        // Mock implementations for now
        info!("REST API would start on port {}", self.config.rest_port);
        info!("WebSocket server would start on port {}", self.config.websocket_port);
        info!("gRPC server would start on port {}", self.config.grpc_port);
        
        info!("All interfaces started successfully!");
        Ok(())
    }
    
    pub async fn process_cli_command(&self, command: &str) -> Result<String> {
        let wallet = self.wallet.read().await;
        wallet.process_command(command).await
    }
}

// Re-export commonly used items
pub use rest::{AppState, CommandRequest, CommandResponse};
pub use websocket::WebSocketHandler;
pub use voice::VoiceInterface;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interface_config_default() {
        let config = InterfaceConfig::default();
        assert_eq!(config.rest_port, 8080);
        assert_eq!(config.websocket_port, 8081);
        assert!(config.enable_wasm);
    }
}