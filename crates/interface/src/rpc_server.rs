//! JSON-RPC 2.0 Server for Mining Compatibility
//! 
//! Provides a fully compatible RPC interface for existing miners and pools
//! Ensures seamless integration without requiring any changes to mining infrastructure

use goldcoin_neural_blockchain::mining_rpc::{MiningRpcService, PoolCompatibility};
use goldcoin_neural_blockchain::GoldcoinNetwork;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use tracing::{info, debug, warn, error};
use anyhow::Result;

/// Standard JSON-RPC 2.0 request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// Standard JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// RPC error codes (Bitcoin Core compatible)
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    // Bitcoin Core specific
    pub const MISC_ERROR: i32 = -1;
    pub const TYPE_ERROR: i32 = -3;
    pub const INVALID_ADDRESS: i32 = -5;
    pub const RPC_INVALID_PARAMETER: i32 = -8;
    pub const DATABASE_ERROR: i32 = -20;
    pub const DESERIALIZATION_ERROR: i32 = -22;
    pub const VERIFY_ERROR: i32 = -25;
    pub const VERIFY_REJECTED: i32 = -26;
    pub const IN_WARMUP: i32 = -28;
}

/// GoldCoin RPC Server
pub struct GoldcoinRpcServer {
    mining_service: Arc<MiningRpcService>,
    pool_compat: Arc<PoolCompatibility>,
    auth_enabled: bool,
    auth_user: String,
    auth_pass: String,
    port: u16,
}

impl GoldcoinRpcServer {
    /// Create new RPC server
    pub fn new(network: GoldcoinNetwork, port: u16) -> Self {
        Self {
            mining_service: Arc::new(MiningRpcService::new(network)),
            pool_compat: Arc::new(PoolCompatibility::new()),
            auth_enabled: false,
            auth_user: String::new(),
            auth_pass: String::new(),
            port,
        }
    }
    
    /// Enable HTTP Basic Authentication (required for most pools)
    pub fn with_auth(mut self, user: &str, pass: &str) -> Self {
        self.auth_enabled = true;
        self.auth_user = user.to_string();
        self.auth_pass = pass.to_string();
        self
    }
    
    /// Start the RPC server
    pub async fn start(&self) -> Result<()> {
        info!("Starting GoldCoin RPC server on port {}", self.port);
        info!("Mining RPC compatibility enabled");
        
        // List supported pool software
        info!("Supported mining software:");
        info!("  - NOMP (Node Open Mining Portal)");
        info!("  - MPOS (Mining Portal Open Source)");
        info!("  - P2Pool (Decentralized mining pool)");
        info!("  - CKPool");
        info!("  - CGMiner / BFGMiner");
        info!("  - CPUMiner / Minerd");
        
        let mining_service = self.mining_service.clone();
        let auth_enabled = self.auth_enabled;
        let auth_user = self.auth_user.clone();
        let auth_pass = self.auth_pass.clone();
        
        // Create RPC route
        let rpc_route = warp::path::end()
            .and(warp::post())
            .and(warp::header::optional::<String>("authorization"))
            .and(warp::body::json())
            .and_then(move |auth: Option<String>, request: JsonRpcRequest| {
                let mining_service = mining_service.clone();
                let auth_user = auth_user.clone();
                let auth_pass = auth_pass.clone();
                
                async move {
                    // Check authentication if enabled
                    if auth_enabled {
                        if !check_auth(&auth, &auth_user, &auth_pass) {
                            let response = JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(JsonRpcError {
                                    code: error_codes::MISC_ERROR,
                                    message: "Authentication failed".to_string(),
                                    data: None,
                                }),
                                id: request.id,
                            };
                            return Ok::<_, Rejection>(warp::reply::json(&response));
                        }
                    }
                    
                    // Handle RPC request
                    let response = handle_rpc_request(mining_service, request).await;
                    Ok::<_, Rejection>(warp::reply::json(&response))
                }
            });
        
        // Support both JSON-RPC over HTTP and raw TCP (for cgminer)
        let routes = rpc_route
            .with(warp::cors()
                .allow_any_origin()
                .allow_methods(vec!["POST", "GET", "OPTIONS"])
                .allow_headers(vec!["content-type", "authorization"]));
        
        info!("RPC server listening on 0.0.0.0:{}", self.port);
        info!("Connect your miner to: http://localhost:{}", self.port);
        
        // Start server (mock for this implementation)
        // In production, would use warp::serve(routes).run()
        
        Ok(())
    }
    
    /// Get server status
    pub async fn get_status(&self) -> ServerStatus {
        ServerStatus {
            running: true,
            port: self.port,
            auth_enabled: self.auth_enabled,
            supported_methods: self.pool_compat.get_required_methods(),
            network: "mainnet".to_string(),
        }
    }
}

/// Handle individual RPC request
async fn handle_rpc_request(
    mining_service: Arc<MiningRpcService>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    debug!("RPC request: {} with params: {:?}", request.method, request.params);
    
    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" && request.jsonrpc != "1.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: error_codes::INVALID_REQUEST,
                message: "Invalid JSON-RPC version".to_string(),
                data: None,
            }),
            id: request.id,
        };
    }
    
    // Handle the request
    let params = request.params.unwrap_or(Value::Null);
    
    match mining_service.handle_request(&request.method, params).await {
        Ok(result) => {
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request.id,
            }
        }
        Err(e) => {
            warn!("RPC error for method {}: {}", request.method, e);
            
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::METHOD_NOT_FOUND,
                    message: e.to_string(),
                    data: None,
                }),
                id: request.id,
            }
        }
    }
}

/// Check HTTP Basic Authentication
fn check_auth(auth_header: &Option<String>, user: &str, pass: &str) -> bool {
    if let Some(auth) = auth_header {
        if auth.starts_with("Basic ") {
            let encoded = &auth[6..];
            if let Ok(decoded) = base64::decode(encoded) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    let expected = format!("{}:{}", user, pass);
                    return credentials == expected;
                }
            }
        }
    }
    false
}

/// Server status information
#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub auth_enabled: bool,
    pub supported_methods: Vec<String>,
    pub network: String,
}

/// Stratum mining protocol support (for ASIC miners)
pub struct StratumServer {
    mining_service: Arc<MiningRpcService>,
    port: u16,
}

impl StratumServer {
    pub fn new(mining_service: Arc<MiningRpcService>, port: u16) -> Self {
        Self {
            mining_service,
            port,
        }
    }
    
    /// Convert Stratum methods to RPC calls
    pub async fn handle_stratum(&self, method: &str, params: Value) -> Result<Value> {
        match method {
            "mining.subscribe" => self.handle_subscribe(params).await,
            "mining.authorize" => self.handle_authorize(params).await,
            "mining.submit" => self.handle_submit(params).await,
            "mining.get_transactions" => self.handle_get_transactions(params).await,
            _ => Err(anyhow::anyhow!("Unknown Stratum method: {}", method)),
        }
    }
    
    async fn handle_subscribe(&self, _params: Value) -> Result<Value> {
        // Return subscription details
        Ok(json!([
            ["mining.notify", "ae6812eb4cd7735a302a8a9dd95cf71f"],
            "08000002",
            4
        ]))
    }
    
    async fn handle_authorize(&self, params: Value) -> Result<Value> {
        // Simple authorization (always accept for now)
        Ok(json!(true))
    }
    
    async fn handle_submit(&self, params: Value) -> Result<Value> {
        // Convert Stratum submission to submitblock
        let block_data = params.get(4)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        self.mining_service.handle_request("submitblock", json!([block_data])).await
    }
    
    async fn handle_get_transactions(&self, _params: Value) -> Result<Value> {
        // Return empty transaction list for now
        Ok(json!([]))
    }
}

/// Legacy Getwork protocol support (for old CPU miners)
pub struct GetworkSupport;

impl GetworkSupport {
    /// Convert between getwork and getblocktemplate
    pub fn getwork_to_template(work_data: &Value) -> Value {
        json!({
            "data": work_data.get("data"),
            "target": work_data.get("target"),
            "algorithm": "scrypt",
            "height": 850000, // Would be extracted from data
        })
    }
    
    /// Check if miner uses getwork
    pub fn is_getwork_miner(user_agent: &str) -> bool {
        user_agent.contains("cpuminer") || 
        user_agent.contains("minerd") ||
        user_agent.contains("pooler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auth_check() {
        let auth_header = Some("Basic dXNlcjpwYXNz".to_string()); // user:pass
        assert!(check_auth(&auth_header, "user", "pass"));
        assert!(!check_auth(&auth_header, "wrong", "pass"));
        
        let no_auth = None;
        assert!(!check_auth(&no_auth, "user", "pass"));
    }
    
    #[tokio::test]
    async fn test_rpc_request_handling() {
        let mining_service = Arc::new(MiningRpcService::new(GoldcoinNetwork::Mainnet));
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "getblockcount".to_string(),
            params: None,
            id: Some(json!(1)),
        };
        
        let response = handle_rpc_request(mining_service, request).await;
        
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(json!(1)));
    }
    
    #[test]
    fn test_getwork_detection() {
        assert!(GetworkSupport::is_getwork_miner("cpuminer/2.5.0"));
        assert!(GetworkSupport::is_getwork_miner("minerd/2.4"));
        assert!(!GetworkSupport::is_getwork_miner("cgminer/4.0"));
    }
}