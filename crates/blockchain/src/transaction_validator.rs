//! GoldCoin Transaction Format Validator
//! 
//! Ensures neural transactions are compatible with GoldCoin's transaction format

use goldcoin_neural_core::{NeuralTransaction, TransactionSemantics};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use sha2::{Sha256, Digest};
use tracing::{info, debug, warn};

/// GoldCoin transaction version numbers
pub const TX_VERSION_1: i32 = 1;
pub const TX_VERSION_2: i32 = 2; // SegWit version (not used by GoldCoin)

/// GoldCoin script opcodes
pub mod opcodes {
    pub const OP_DUP: u8 = 0x76;
    pub const OP_HASH160: u8 = 0xa9;
    pub const OP_EQUALVERIFY: u8 = 0x88;
    pub const OP_CHECKSIG: u8 = 0xac;
    pub const OP_RETURN: u8 = 0x6a;
}

/// GoldCoin address prefixes
pub mod address_prefix {
    pub const MAINNET_P2PKH: u8 = 32;  // 'G' addresses
    pub const MAINNET_P2SH: u8 = 5;    // '3' addresses (multisig)
    pub const TESTNET_P2PKH: u8 = 111; // 'm' or 'n' addresses
    pub const TESTNET_P2SH: u8 = 196;  // '2' addresses
}

/// GoldCoin transaction input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub prev_txid: String,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

/// GoldCoin transaction output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: i64, // Satoshis
    pub script_pubkey: Vec<u8>,
}

/// Complete GoldCoin transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldcoinTransaction {
    pub version: i32,
    pub locktime: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl GoldcoinTransaction {
    /// Calculate transaction ID (double SHA256 of serialized tx)
    pub fn txid(&self) -> String {
        let serialized = self.serialize();
        let hash1 = Sha256::digest(&serialized);
        let hash2 = Sha256::digest(&hash1);
        
        // Reverse bytes for display (little-endian)
        let mut txid_bytes = hash2.to_vec();
        txid_bytes.reverse();
        
        hex::encode(txid_bytes)
    }
    
    /// Serialize transaction for hashing
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Version (4 bytes, little-endian)
        data.extend_from_slice(&self.version.to_le_bytes());
        
        // Input count (varint)
        data.extend_from_slice(&encode_varint(self.inputs.len() as u64));
        
        // Inputs
        for input in &self.inputs {
            // Previous output hash (32 bytes)
            if let Ok(txid_bytes) = hex::decode(&input.prev_txid) {
                data.extend_from_slice(&txid_bytes);
            }
            // Previous output index (4 bytes)
            data.extend_from_slice(&input.vout.to_le_bytes());
            // Script length (varint)
            data.extend_from_slice(&encode_varint(input.script_sig.len() as u64));
            // Script
            data.extend_from_slice(&input.script_sig);
            // Sequence (4 bytes)
            data.extend_from_slice(&input.sequence.to_le_bytes());
        }
        
        // Output count (varint)
        data.extend_from_slice(&encode_varint(self.outputs.len() as u64));
        
        // Outputs
        for output in &self.outputs {
            // Value (8 bytes)
            data.extend_from_slice(&output.value.to_le_bytes());
            // Script length (varint)
            data.extend_from_slice(&encode_varint(output.script_pubkey.len() as u64));
            // Script
            data.extend_from_slice(&output.script_pubkey);
        }
        
        // Locktime (4 bytes)
        data.extend_from_slice(&self.locktime.to_le_bytes());
        
        data
    }
    
    /// Calculate transaction size in bytes
    pub fn size(&self) -> usize {
        self.serialize().len()
    }
    
    /// Calculate transaction weight (GoldCoin doesn't use SegWit weight)
    pub fn weight(&self) -> usize {
        self.size() * 4 // Non-SegWit weight calculation
    }
    
    /// Validate transaction format
    pub fn validate(&self) -> Result<()> {
        // Check version
        if self.version != TX_VERSION_1 {
            return Err(anyhow::anyhow!("Invalid transaction version: {}", self.version));
        }
        
        // Must have at least one input
        if self.inputs.is_empty() {
            return Err(anyhow::anyhow!("Transaction must have at least one input"));
        }
        
        // Must have at least one output
        if self.outputs.is_empty() {
            return Err(anyhow::anyhow!("Transaction must have at least one output"));
        }
        
        // Check for negative output values
        for (i, output) in self.outputs.iter().enumerate() {
            if output.value < 0 {
                return Err(anyhow::anyhow!("Output {} has negative value", i));
            }
        }
        
        // Check transaction size (max 32MB for GoldCoin)
        let size = self.size();
        if size > 32 * 1024 * 1024 {
            return Err(anyhow::anyhow!("Transaction too large: {} bytes", size));
        }
        
        Ok(())
    }
}

/// Transaction format converter
pub struct TransactionConverter {
    network: Network,
}

#[derive(Debug, Clone, Copy)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl TransactionConverter {
    pub fn new(network: Network) -> Self {
        Self { network }
    }
    
    /// Convert neural transaction to GoldCoin format
    pub fn from_neural(&self, neural_tx: &NeuralTransaction) -> Result<GoldcoinTransaction> {
        debug!("Converting neural transaction {} to GoldCoin format", neural_tx.id);
        
        // For this mock implementation, create a simple transaction
        // In production, this would interface with actual UTXO selection
        
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        
        // Mock input (would come from UTXO selection)
        inputs.push(TxInput {
            prev_txid: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            vout: 0,
            script_sig: vec![], // Would be filled with signature
            sequence: 0xfffffffe, // RBF enabled
        });
        
        // Create output for recipient
        let recipient_script = self.create_p2pkh_script(&neural_tx.semantics.to)?;
        let amount_satoshis = (neural_tx.semantics.amount * 100_000_000.0) as i64;
        
        outputs.push(TxOutput {
            value: amount_satoshis,
            script_pubkey: recipient_script,
        });
        
        // Add OP_RETURN output for memo if present
        if let Some(memo) = &neural_tx.semantics.memo {
            let op_return_script = self.create_op_return_script(memo)?;
            outputs.push(TxOutput {
                value: 0,
                script_pubkey: op_return_script,
            });
        }
        
        let tx = GoldcoinTransaction {
            version: TX_VERSION_1,
            locktime: 0,
            inputs,
            outputs,
        };
        
        tx.validate()?;
        
        info!("Created GoldCoin transaction with {} inputs, {} outputs", 
              tx.inputs.len(), tx.outputs.len());
        
        Ok(tx)
    }
    
    /// Create P2PKH (Pay to Public Key Hash) script
    fn create_p2pkh_script(&self, address: &str) -> Result<Vec<u8>> {
        // This is a simplified version - real implementation would decode the address
        let mut script = Vec::new();
        
        script.push(opcodes::OP_DUP);
        script.push(opcodes::OP_HASH160);
        script.push(20); // Push 20 bytes
        
        // Mock public key hash (would be decoded from address)
        script.extend_from_slice(&[0u8; 20]);
        
        script.push(opcodes::OP_EQUALVERIFY);
        script.push(opcodes::OP_CHECKSIG);
        
        Ok(script)
    }
    
    /// Create OP_RETURN script for data storage
    fn create_op_return_script(&self, data: &str) -> Result<Vec<u8>> {
        let data_bytes = data.as_bytes();
        
        if data_bytes.len() > 80 {
            return Err(anyhow::anyhow!("OP_RETURN data too large: {} bytes", data_bytes.len()));
        }
        
        let mut script = Vec::new();
        script.push(opcodes::OP_RETURN);
        script.push(data_bytes.len() as u8);
        script.extend_from_slice(data_bytes);
        
        Ok(script)
    }
    
    /// Estimate transaction fee based on size and fee rate
    pub fn estimate_fee(&self, tx: &GoldcoinTransaction, fee_rate: f64) -> f64 {
        let size = tx.size();
        let fee_satoshis = (size as f64 * fee_rate) as i64;
        fee_satoshis as f64 / 100_000_000.0 // Convert to GLD
    }
}

/// Encode integer as Bitcoin varint
fn encode_varint(n: u64) -> Vec<u8> {
    match n {
        0..=0xfc => vec![n as u8],
        0xfd..=0xffff => {
            let mut v = vec![0xfd];
            v.extend_from_slice(&(n as u16).to_le_bytes());
            v
        }
        0x10000..=0xffffffff => {
            let mut v = vec![0xfe];
            v.extend_from_slice(&(n as u32).to_le_bytes());
            v
        }
        _ => {
            let mut v = vec![0xff];
            v.extend_from_slice(&n.to_le_bytes());
            v
        }
    }
}

/// Validate GoldCoin address format
pub fn validate_address(address: &str, network: Network) -> Result<()> {
    // Basic validation - real implementation would use base58check
    if address.is_empty() {
        return Err(anyhow::anyhow!("Empty address"));
    }
    
    let first_char = address.chars().next().unwrap();
    
    match network {
        Network::Mainnet => {
            if first_char != 'G' && first_char != '3' {
                return Err(anyhow::anyhow!("Invalid mainnet address prefix"));
            }
        }
        Network::Testnet => {
            if first_char != 'm' && first_char != 'n' && first_char != '2' {
                return Err(anyhow::anyhow!("Invalid testnet address prefix"));
            }
        }
    }
    
    // Check length (simplified)
    if address.len() < 26 || address.len() > 35 {
        return Err(anyhow::anyhow!("Invalid address length"));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_serialization() {
        let tx = GoldcoinTransaction {
            version: 1,
            locktime: 0,
            inputs: vec![TxInput {
                prev_txid: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                vout: 0,
                script_sig: vec![],
                sequence: 0xfffffffe,
            }],
            outputs: vec![TxOutput {
                value: 100000000, // 1 GLD
                script_pubkey: vec![0x76, 0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
                                   0x88, 0xac],
            }],
        };
        
        assert!(tx.validate().is_ok());
        assert!(tx.size() > 0);
        assert!(!tx.txid().is_empty());
    }
    
    #[test]
    fn test_address_validation() {
        // Mainnet addresses
        assert!(validate_address("GXxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", Network::Mainnet).is_ok());
        assert!(validate_address("3Xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", Network::Mainnet).is_ok());
        
        // Testnet addresses
        assert!(validate_address("mXxxxxxxxxxxxxxxxxxxxxxxxxxxxxXx", Network::Testnet).is_ok());
        assert!(validate_address("nXxxxxxxxxxxxxxxxxxxxxxxxxxxxxXx", Network::Testnet).is_ok());
        
        // Invalid addresses
        assert!(validate_address("", Network::Mainnet).is_err());
        assert!(validate_address("1BitcoinAddress", Network::Mainnet).is_err());
        assert!(validate_address("Short", Network::Mainnet).is_err());
    }
    
    #[test]
    fn test_op_return_script() {
        let converter = TransactionConverter::new(Network::Mainnet);
        
        // Valid OP_RETURN
        let script = converter.create_op_return_script("Hello GoldCoin").unwrap();
        assert_eq!(script[0], opcodes::OP_RETURN);
        
        // Too large OP_RETURN
        let large_data = "x".repeat(100);
        assert!(converter.create_op_return_script(&large_data).is_err());
    }
}