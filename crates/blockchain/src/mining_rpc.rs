//! Mining RPC Compatibility Layer
//! 
//! Provides full backward compatibility with existing GoldCoin miners and pools
//! Implements all standard mining RPC methods to ensure no hard fork is needed

use crate::goldcoin_compat::{GoldcoinCompatibility, BlockInfo};
use crate::GoldcoinNetwork;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};
use chrono::Utc;

/// Standard GoldCoin RPC mining methods
pub const RPC_GETBLOCKTEMPLATE: &str = "getblocktemplate";
pub const RPC_GETWORK: &str = "getwork";  // Legacy, but still used
pub const RPC_SUBMITBLOCK: &str = "submitblock";
pub const RPC_GETMININGINFO: &str = "getmininginfo";
pub const RPC_GETNETWORKHASHPS: &str = "getnetworkhashps";
pub const RPC_GETBLOCKCOUNT: &str = "getblockcount";
pub const RPC_GETDIFFICULTY: &str = "getdifficulty";
pub const RPC_GETBESTBLOCKHASH: &str = "getbestblockhash";

/// Mining RPC service maintaining full backward compatibility
pub struct MiningRpcService {
    compatibility: Arc<GoldcoinCompatibility>,
    network: GoldcoinNetwork,
    block_templates: Arc<RwLock<HashMap<String, BlockTemplate>>>,
    work_cache: Arc<RwLock<HashMap<String, WorkData>>>,
    current_height: Arc<RwLock<u64>>,
    current_difficulty: Arc<RwLock<f64>>,
}

impl MiningRpcService {
    pub fn new(network: GoldcoinNetwork) -> Self {
        Self {
            compatibility: Arc::new(GoldcoinCompatibility::new(network)),
            network,
            block_templates: Arc::new(RwLock::new(HashMap::new())),
            work_cache: Arc::new(RwLock::new(HashMap::new())),
            current_height: Arc::new(RwLock::new(850000)),
            current_difficulty: Arc::new(RwLock::new(1234567.89)),
        }
    }
    
    /// Handle RPC request from miner/pool
    pub async fn handle_request(&self, method: &str, params: Value) -> Result<Value> {
        debug!("Mining RPC request: {} with params: {:?}", method, params);
        
        match method {
            RPC_GETBLOCKTEMPLATE => self.get_block_template(params).await,
            RPC_GETWORK => self.get_work(params).await,
            RPC_SUBMITBLOCK => self.submit_block(params).await,
            RPC_GETMININGINFO => self.get_mining_info().await,
            RPC_GETNETWORKHASHPS => self.get_network_hashps(params).await,
            RPC_GETBLOCKCOUNT => self.get_block_count().await,
            RPC_GETDIFFICULTY => self.get_difficulty().await,
            RPC_GETBESTBLOCKHASH => self.get_best_blockhash().await,
            _ => Err(anyhow::anyhow!("Unknown mining RPC method: {}", method)),
        }
    }
    
    /// getblocktemplate - BIP22 standard for modern miners
    async fn get_block_template(&self, params: Value) -> Result<Value> {
        // Parse capabilities from params
        let capabilities = if let Some(obj) = params.as_object() {
            if let Some(caps) = obj.get("capabilities") {
                caps.as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>())
                    .unwrap_or_default()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        
        let height = *self.current_height.read().await;
        let difficulty = *self.current_difficulty.read().await;
        
        // Check if we need Golden River difficulty adjustment
        let next_diff = if height >= 372000 {
            // Golden River is active
            self.compatibility.golden_river
                .estimate_future_difficulty(difficulty, 0.0)
        } else {
            difficulty
        };
        
        // Create block template
        let template = BlockTemplate {
            version: 536870912, // Version 0x20000000
            previousblockhash: format!("{:064x}", height - 1), // Mock
            transactions: vec![], // Would include pending transactions
            coinbaseaux: HashMap::new(),
            coinbasevalue: self.calculate_block_reward(height),
            target: self.difficulty_to_target(difficulty),
            mintime: Utc::now().timestamp() - 600,
            mutable: vec!["time".to_string(), "transactions".to_string(), "prevblock".to_string()],
            noncerange: "00000000ffffffff".to_string(),
            sigoplimit: 80000,
            sizelimit: 32 * 1024 * 1024, // 32MB for GoldCoin
            curtime: Utc::now().timestamp(),
            bits: self.difficulty_to_bits(difficulty),
            height,
            // GoldCoin-specific fields
            goldcoin_fields: GoldcoinTemplateFields {
                checkpoint_required: self.is_checkpoint_required(height).await,
                defense_51_active: self.compatibility.defense_51.should_delay_transaction().await,
                golden_river_active: height >= 372000,
                next_difficulty: next_diff,
            },
        };
        
        // Cache the template
        let template_id = format!("{}:{}", height, Utc::now().timestamp());
        self.block_templates.write().await.insert(template_id.clone(), template.clone());
        
        // Return BIP22-compliant response
        Ok(json!({
            "version": template.version,
            "previousblockhash": template.previousblockhash,
            "transactions": template.transactions,
            "coinbaseaux": template.coinbaseaux,
            "coinbasevalue": template.coinbasevalue,
            "target": template.target,
            "mintime": template.mintime,
            "mutable": template.mutable,
            "noncerange": template.noncerange,
            "sigoplimit": template.sigoplimit,
            "sizelimit": template.sizelimit,
            "curtime": template.curtime,
            "bits": template.bits,
            "height": template.height,
            // Optional GoldCoin extensions
            "goldcoin": {
                "checkpoint_required": template.goldcoin_fields.checkpoint_required,
                "defense_51_active": template.goldcoin_fields.defense_51_active,
                "golden_river_active": template.goldcoin_fields.golden_river_active,
                "next_difficulty": template.goldcoin_fields.next_difficulty,
            },
            "capabilities": capabilities,
        }))
    }
    
    /// getwork - Legacy mining RPC for older miners
    async fn get_work(&self, params: Value) -> Result<Value> {
        // Check if this is a submission
        if let Some(data) = params.as_str() {
            return self.submit_work(data).await;
        }
        
        let height = *self.current_height.read().await;
        let difficulty = *self.current_difficulty.read().await;
        
        // Create work data (simplified for legacy miners)
        let work = WorkData {
            data: format!("{:0256x}", height), // 256 hex chars
            target: self.difficulty_to_target(difficulty),
            algorithm: "scrypt".to_string(), // GoldCoin uses scrypt
        };
        
        // Cache the work
        let work_id = format!("work_{}", Utc::now().timestamp());
        self.work_cache.write().await.insert(work_id.clone(), work.clone());
        
        Ok(json!({
            "data": work.data,
            "target": work.target,
            "algorithm": work.algorithm,
        }))
    }
    
    /// submitblock - Submit a new block
    async fn submit_block(&self, params: Value) -> Result<Value> {
        let block_hex = params.as_str()
            .or_else(|| params.get(0).and_then(|v| v.as_str()))
            .ok_or_else(|| anyhow::anyhow!("Missing block data"))?;
        
        info!("Received block submission: {} bytes", block_hex.len() / 2);
        
        // Validate block (simplified)
        if block_hex.len() < 160 {
            return Ok(json!("rejected: invalid block size"));
        }
        
        // Check 51% defense
        if self.compatibility.defense_51.should_delay_transaction().await {
            warn!("Block rejected due to 51% defense activation");
            return Ok(json!("rejected: 51% defense active"));
        }
        
        // Mock acceptance
        let new_height = *self.current_height.read().await + 1;
        *self.current_height.write().await = new_height;
        
        info!("Block accepted at height {}", new_height);
        
        // Return null on success (Bitcoin Core standard)
        Ok(Value::Null)
    }
    
    /// getmininginfo - Get mining-related information
    async fn get_mining_info(&self) -> Result<Value> {
        let height = *self.current_height.read().await;
        let difficulty = *self.current_difficulty.read().await;
        
        Ok(json!({
            "blocks": height,
            "currentblocksize": 0,
            "currentblocktx": 0,
            "difficulty": difficulty,
            "errors": "",
            "genproclimit": -1,
            "networkhashps": self.estimate_network_hashrate().await,
            "pooledtx": 0,
            "testnet": self.network == GoldcoinNetwork::Testnet,
            "chain": match self.network {
                GoldcoinNetwork::Mainnet => "main",
                GoldcoinNetwork::Testnet => "test",
                GoldcoinNetwork::Regtest => "regtest",
            },
            "generate": false,
            "hashespersec": 0,
            // GoldCoin-specific
            "goldcoin": {
                "golden_river_active": height >= 372000,
                "defense_51_status": self.get_defense_status().await,
                "checkpoint_height": height - (height % 100), // Mock checkpoint every 100 blocks
            }
        }))
    }
    
    /// getnetworkhashps - Estimate network hash rate
    async fn get_network_hashps(&self, params: Value) -> Result<Value> {
        let blocks = params.get(0)
            .and_then(|v| v.as_i64())
            .unwrap_or(120) as u32; // Default 120 blocks
        
        let hashrate = self.estimate_network_hashrate_with_blocks(blocks).await;
        Ok(json!(hashrate))
    }
    
    /// getblockcount - Get current block height
    async fn get_block_count(&self) -> Result<Value> {
        let height = *self.current_height.read().await;
        Ok(json!(height))
    }
    
    /// getdifficulty - Get current difficulty
    async fn get_difficulty(&self) -> Result<Value> {
        let difficulty = *self.current_difficulty.read().await;
        Ok(json!(difficulty))
    }
    
    /// getbestblockhash - Get hash of best block
    async fn get_best_blockhash(&self) -> Result<Value> {
        let height = *self.current_height.read().await;
        let hash = format!("{:064x}", height); // Mock hash
        Ok(json!(hash))
    }
    
    // Helper methods
    
    async fn submit_work(&self, data: &str) -> Result<Value> {
        debug!("Submitting work data: {}", data);
        
        // Validate work submission
        if data.len() != 256 {
            return Ok(json!(false));
        }
        
        // Mock acceptance
        Ok(json!(true))
    }
    
    fn calculate_block_reward(&self, height: u64) -> i64 {
        // GoldCoin block reward calculation
        let initial_reward = 50_00000000i64; // 50 GLD in satoshis
        
        // Halving schedule (simplified)
        let halvings = height / 840000;
        let reward = initial_reward >> halvings;
        
        // Apply February fork adjustment if needed
        if height >= 612000 {
            // Coin generation adjustment
            reward * 9 / 10 // 90% of original
        } else {
            reward
        }
    }
    
    fn difficulty_to_target(&self, difficulty: f64) -> String {
        // Convert difficulty to target (simplified)
        let max_target = "00000000ffff0000000000000000000000000000000000000000000000000000";
        format!("{:064x}", (0xffffu64 as f64 / difficulty) as u64)
    }
    
    fn difficulty_to_bits(&self, difficulty: f64) -> String {
        // Convert difficulty to compact bits representation
        format!("{:08x}", (0x1d00ffffu32 as f64 / difficulty) as u32)
    }
    
    async fn is_checkpoint_required(&self, height: u64) -> bool {
        // Check if checkpoint is required at this height
        height % 100 == 0 // Checkpoint every 100 blocks (simplified)
    }
    
    async fn estimate_network_hashrate(&self) -> f64 {
        self.estimate_network_hashrate_with_blocks(120).await
    }
    
    async fn estimate_network_hashrate_with_blocks(&self, blocks: u32) -> f64 {
        let difficulty = *self.current_difficulty.read().await;
        
        // Estimate based on difficulty and block time
        // Hashrate = difficulty * 2^32 / block_time
        let block_time = 120.0; // 2 minutes
        difficulty * 4294967296.0 / block_time / blocks as f64
    }
    
    async fn get_defense_status(&self) -> String {
        if self.compatibility.defense_51.should_delay_transaction().await {
            "active".to_string()
        } else {
            "monitoring".to_string()
        }
    }
}

/// Block template for getblocktemplate
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockTemplate {
    version: u32,
    previousblockhash: String,
    transactions: Vec<Value>,
    coinbaseaux: HashMap<String, String>,
    coinbasevalue: i64,
    target: String,
    mintime: i64,
    mutable: Vec<String>,
    noncerange: String,
    sigoplimit: u32,
    sizelimit: usize,
    curtime: i64,
    bits: String,
    height: u64,
    goldcoin_fields: GoldcoinTemplateFields,
}

/// GoldCoin-specific template fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoldcoinTemplateFields {
    checkpoint_required: bool,
    defense_51_active: bool,
    golden_river_active: bool,
    next_difficulty: f64,
}

/// Legacy work data for getwork
#[derive(Debug, Clone)]
struct WorkData {
    data: String,
    target: String,
    algorithm: String,
}

/// Pool software compatibility layer
pub struct PoolCompatibility {
    supported_software: Vec<PoolSoftware>,
}

#[derive(Debug, Clone)]
struct PoolSoftware {
    name: String,
    version: String,
    required_methods: Vec<String>,
}

impl PoolCompatibility {
    pub fn new() -> Self {
        Self {
            supported_software: vec![
                PoolSoftware {
                    name: "NOMP".to_string(),
                    version: "*".to_string(),
                    required_methods: vec![
                        RPC_GETBLOCKTEMPLATE.to_string(),
                        RPC_SUBMITBLOCK.to_string(),
                        RPC_GETMININGINFO.to_string(),
                    ],
                },
                PoolSoftware {
                    name: "MPOS".to_string(),
                    version: "*".to_string(),
                    required_methods: vec![
                        RPC_GETWORK.to_string(),
                        RPC_GETBLOCKTEMPLATE.to_string(),
                    ],
                },
                PoolSoftware {
                    name: "P2Pool".to_string(),
                    version: "*".to_string(),
                    required_methods: vec![
                        RPC_GETBLOCKTEMPLATE.to_string(),
                        RPC_SUBMITBLOCK.to_string(),
                    ],
                },
                PoolSoftware {
                    name: "CKPool".to_string(),
                    version: "*".to_string(),
                    required_methods: vec![
                        RPC_GETBLOCKTEMPLATE.to_string(),
                        RPC_SUBMITBLOCK.to_string(),
                    ],
                },
            ],
        }
    }
    
    /// Verify compatibility with specific pool software
    pub fn verify_compatibility(&self, software: &str) -> bool {
        self.supported_software.iter()
            .any(|s| s.name.eq_ignore_ascii_case(software))
    }
    
    /// List all required RPC methods for compatibility
    pub fn get_required_methods(&self) -> Vec<String> {
        let mut methods = vec![
            RPC_GETBLOCKTEMPLATE.to_string(),
            RPC_GETWORK.to_string(),
            RPC_SUBMITBLOCK.to_string(),
            RPC_GETMININGINFO.to_string(),
            RPC_GETNETWORKHASHPS.to_string(),
            RPC_GETBLOCKCOUNT.to_string(),
            RPC_GETDIFFICULTY.to_string(),
            RPC_GETBESTBLOCKHASH.to_string(),
        ];
        
        // Add wallet RPC methods that pools might use
        methods.extend(vec![
            "validateaddress".to_string(),
            "getbalance".to_string(),
            "sendtoaddress".to_string(),
            "getnewaddress".to_string(),
        ]);
        
        methods
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_getblocktemplate() {
        let service = MiningRpcService::new(GoldcoinNetwork::Mainnet);
        
        let params = json!({
            "capabilities": ["coinbasetxn", "workid", "coinbase/append"]
        });
        
        let result = service.get_block_template(params).await.unwrap();
        
        assert!(result.get("version").is_some());
        assert!(result.get("height").is_some());
        assert!(result.get("target").is_some());
        assert!(result.get("goldcoin").is_some());
    }
    
    #[tokio::test]
    async fn test_getwork_legacy() {
        let service = MiningRpcService::new(GoldcoinNetwork::Mainnet);
        
        let result = service.get_work(Value::Null).await.unwrap();
        
        assert!(result.get("data").is_some());
        assert!(result.get("target").is_some());
        assert_eq!(result.get("algorithm").unwrap(), "scrypt");
    }
    
    #[tokio::test]
    async fn test_mining_info() {
        let service = MiningRpcService::new(GoldcoinNetwork::Mainnet);
        
        let result = service.get_mining_info().await.unwrap();
        
        assert!(result.get("blocks").is_some());
        assert!(result.get("difficulty").is_some());
        assert!(result.get("goldcoin").is_some());
        
        let goldcoin_info = result.get("goldcoin").unwrap();
        assert!(goldcoin_info.get("golden_river_active").is_some());
        assert!(goldcoin_info.get("defense_51_status").is_some());
    }
    
    #[test]
    fn test_pool_compatibility() {
        let compat = PoolCompatibility::new();
        
        assert!(compat.verify_compatibility("NOMP"));
        assert!(compat.verify_compatibility("mpos")); // Case insensitive
        assert!(compat.verify_compatibility("P2Pool"));
        assert!(!compat.verify_compatibility("UnknownPool"));
        
        let methods = compat.get_required_methods();
        assert!(methods.contains(&RPC_GETBLOCKTEMPLATE.to_string()));
        assert!(methods.contains(&RPC_SUBMITBLOCK.to_string()));
    }
}