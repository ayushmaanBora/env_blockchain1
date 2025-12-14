use crate::transaction::{Transaction, TaskStatus};
use crate::wallet::{Wallet, WalletManager};
use crate::marketplace::Marketplace;
use crate::utils::hash_data;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;

const CHAIN_FILE: &str = "chain.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let hash = hash_data(&format!("{}{}{:?}{}", index, timestamp, transactions, previous_hash));
        Self { index, timestamp, transactions, previous_hash, hash }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    Block(Block),
    Transaction(Transaction),
    ValidationResult(String, TaskStatus),
}

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    #[serde(skip)]
    pub wallets: WalletManager,
    #[serde(skip)]
    pub marketplace: Marketplace,
    
    pub stake_amount: u64,
    pub tasks_for_validation: Vec<Transaction>,
    pub tasks_for_mining: Vec<Transaction>,
    
    // INDUSTRIAL SECURITY
    pub authorized_sentinels: HashSet<String>, // Whitelist of Factory IoT Nodes
    pub used_signatures: HashSet<String>,      // Anti-Replay Database
}

impl Blockchain {
    pub fn new() -> Self {
        // Load or Genesis...
        if let Ok(data) = fs::read_to_string(CHAIN_FILE) {
            if let Ok(mut loaded_chain) = serde_json::from_str::<Blockchain>(&data) {
                loaded_chain.wallets = WalletManager::new();
                loaded_chain.marketplace = Marketplace::new();
                println!("🏭 Industrial Ledger Loaded.");
                return loaded_chain;
            }
        }

        let genesis_block = Block::new(0, vec![], "0".to_string());
        
        let mut authorized_sentinels = HashSet::new();
        // Pre-approve a "Factory Sentinel" for testing
        authorized_sentinels.insert("yuki-industrial-01".to_string());
        
        Self {
            chain: vec![genesis_block],
            wallets: WalletManager::new(),
            marketplace: Marketplace::new(),
            stake_amount: 500, // Higher stake for Corporations
            tasks_for_validation: Vec::new(),
            tasks_for_mining: Vec::new(),
            authorized_sentinels,
            used_signatures: HashSet::new(),
        }
    }

    pub fn save_chain(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write(CHAIN_FILE, data);
        }
    }

    // --- INDUSTRIAL REWARD LOGIC ---
    fn calculate_industrial_credit(&self, metadata_json: &str) -> u64 {
        let v: Value = match serde_json::from_str(metadata_json) {
            Ok(val) => val,
            Err(_) => return 0,
        };

        match v["type"].as_str() {
            Some("carbon_capture") => {
                let tons = v["tons_captured"].as_f64().unwrap_or(0.0);
                // 1 Ton Captured = 100 Yuki Credits
                (tons * 100.0) as u64
            },
            Some("wastewater_treatment") => {
                let liters = v["liters_treated"].as_u64().unwrap_or(0);
                // 1000 Liters = 1 Yuki Credit
                liters / 1000 
            },
            _ => 0,
        }
    }

    // --- THE "EARN-TO-EMIT" VALIDATOR ---
    fn validate_industrial_packet(&self, metadata: &Value) -> (bool, String) {
        
        // 1. HARDWARE ORIGIN (Sentinel Check)
        if let Some(id) = metadata["sentinel_id"].as_str() {
            if !self.authorized_sentinels.contains(id) {
                return (false, format!("⚠️ UNAUTHORIZED HARDWARE: Node '{}' is not in the Industrial Registry.", id));
            }
        } else {
            return (false, "⚠️ INVALID PACKET: Missing Sentinel ID.".to_string());
        }

        // 2. ANTI-REPLAY (The Chlorophyll/Sensor Loop Fix)
        if let Some(sig) = metadata["hardware_signature"].as_str() {
            if self.used_signatures.contains(sig) {
                return (false, "🚨 FRAUD ALERT: Replay Attack. This sensor packet was already used.".to_string());
            }
        } else {
            return (false, "⚠️ INVALID PACKET: Missing Hardware Signature.".to_string());
        }

        // 3. ANOMALY DETECTION (Industrial Physics)
        if let Some(task_type) = metadata["type"].as_str() {
            if task_type == "carbon_capture" {
                 if let Some(tons) = metadata["tons_captured"].as_f64() {
                     // Physics check: A single unit typically captures max 50 tons/hour
                     if tons > 50.0 {
                         return (false, "⚠️ ANOMALY: Reported capture rate exceeds physical limits of equipment.".to_string());
                     }
                 }
            }
        }

        (true, "✅ Industrial Compliance Verified.".to_string())
    }

    // --- SUBMIT WORK (EARN) ---
    pub fn submit_industrial_task(&mut self, wallet_address: &str, task_name: String, proof_metadata: String) -> Option<Transaction> {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            
            // Stake Check (Corporations stake more)
            if wallet.balance_yuki < self.stake_amount {
                println!("❌ INSUFFICIENT COLLATERAL. Operations Halted.");
                return None;
            }
            wallet.balance_yuki -= self.stake_amount;

            let credit = self.calculate_industrial_credit(&proof_metadata);
            
            let transaction = Transaction::new(
                wallet_address.to_string(),
                "Protocol-Mint".to_string(), 
                credit,
                task_name,
                proof_metadata,
            );
            
            self.tasks_for_validation.push(transaction.clone());
            self.save_chain();
            self.wallets.save_wallets();
            Some(transaction)
        } else {
            None
        }
    }

    // --- BURN TO EMIT (SPEND) ---
    // This is the ONLY way tokens leave a wallet. No transfers.
    pub fn request_emission_permit(&mut self, wallet_address: &str, tons_to_emit: u64) -> bool {
        let cost_per_ton = 100; // 1 Ton Emission costs 100 Yuki (Ratio 1:1 with Capture)
        let total_cost = tons_to_emit * cost_per_ton;

        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki >= total_cost {
                // BURN THE TOKENS
                wallet.balance_yuki -= total_cost;
                println!("🔥 BURN SUCCESSFUL: {} Yuki Incinerated.", total_cost);
                println!("🏭 PERMIT GRANTED: Authorized to emit {} tons CO2.", tons_to_emit);
                
                // In a real device, this would send an MQTT signal to unlock the smokestack valve
                self.wallets.save_wallets();
                return true;
            } else {
                println!("❌ PERMIT DENIED: Insufficient Impact Credits.");
                println!("   Required: {} | Available: {}", total_cost, wallet.balance_yuki);
                println!("   ACTION: Halt Emissions or perform Restoration immediately.");
                return false;
            }
        }
        false
    }

    pub fn run_automated_validation(&mut self) -> Vec<(String, TaskStatus)> {
        let mut results = Vec::new();
        for i in (0..self.tasks_for_validation.len()).rev() {
            let task = self.tasks_for_validation[i].clone();
            let mut is_valid = false;
            let mut reason = "Unknown".to_string();

            if let Ok(v) = serde_json::from_str::<Value>(&task.proof_metadata) {
                let (passed, msg) = self.validate_industrial_packet(&v);
                is_valid = passed;
                reason = msg;
                
                // If valid, LOCK the signature forever
                if passed { 
                    if let Some(sig) = v["hardware_signature"].as_str() { 
                        self.used_signatures.insert(sig.to_string()); 
                    } 
                }
            }
            
            if is_valid {
                println!("\n[SENTINEL] Packet {} APPROVED: {}", task.task, reason);
                let mut validated_task = self.tasks_for_validation.remove(i);
                validated_task.status = TaskStatus::Validated;
                self.tasks_for_mining.push(validated_task);
                results.push((task.task, TaskStatus::Validated));
            } else {
                println!("\n[SENTINEL] Packet {} REJECTED: {}", task.task, reason);
                let _ = self.tasks_for_validation.remove(i);
                results.push((task.task, TaskStatus::Rejected));
                self.wallets.save_wallets(); // Slashing happens here (stake already removed)
            }
        }
        self.save_chain();
        results
    }

    pub fn mine_block(&mut self) -> Option<Block> {
        if self.tasks_for_mining.is_empty() { return None; }
        let mut transactions_for_block = Vec::new();
        while let Some(mut task) = self.tasks_for_mining.pop() {
            if let Some(wallet) = self.wallets.get_mut_wallet(&task.sender) {
                wallet.balance_yuki += task.amount + self.stake_amount; // Return Stake + Reward
                task.receiver = task.sender.clone();
                transactions_for_block.push(task);
            }
        }
        if transactions_for_block.is_empty() { return None; }
        let previous_block = self.chain.last().unwrap();
        let new_block = Block::new(previous_block.index + 1, transactions_for_block, previous_block.hash.clone());
        println!("✅ New Industrial Block {} mined!", new_block.hash);
        self.chain.push(new_block.clone());
        self.save_chain();
        self.wallets.save_wallets();
        Some(new_block)
    }

    // --- Helpers (Network Sync, Wallets) ---
    pub fn create_wallet(&mut self) -> Wallet { let w = self.wallets.create_wallet(); self.wallets.save_wallets(); w }
    pub fn view_wallets(&self) { self.wallets.view_wallets(); }
    pub fn add_block_from_network(&mut self, block: Block) { /* Same as before, just update pools */ 
        let previous_block = self.chain.last().unwrap();
        if block.previous_hash == previous_block.hash {
            for tx in &block.transactions {
                self.tasks_for_mining.retain(|t| t.task != tx.task);
                self.tasks_for_validation.retain(|t| t.task != tx.task);
            }
            self.chain.push(block);
            self.save_chain();
            self.wallets.save_wallets(); 
        }
    }
    pub fn add_task_from_network(&mut self, tx: Transaction) { if !self.tasks_for_validation.iter().any(|t| t.task == tx.task) { self.tasks_for_validation.push(tx); } }
    pub fn update_task_status_from_network(&mut self, task_id: &str, status: TaskStatus) { 
        if let Some(pos) = self.tasks_for_validation.iter().position(|t| t.task == task_id) {
            match status {
                TaskStatus::Validated => { let t = self.tasks_for_validation.remove(pos); self.tasks_for_mining.push(t); }
                TaskStatus::Rejected => { self.tasks_for_validation.remove(pos); }
                _ => {}
            }
        }
    }
}