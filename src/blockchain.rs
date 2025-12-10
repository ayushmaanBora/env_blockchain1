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
    // We skip saving these fields because they manage their own files (wallets.json) or are volatile
    #[serde(skip)]
    pub wallets: WalletManager,
    #[serde(skip)]
    pub marketplace: Marketplace,
    
    pub stake_amount: u64,
    pub tasks_for_validation: Vec<Transaction>,
    pub tasks_for_mining: Vec<Transaction>,
    
    // These need to be saved to maintain security state
    pub authorized_devices: HashSet<String>, 
    pub used_evidence_urls: HashSet<String>, 
}

impl Blockchain {
    pub fn new() -> Self {
        // 1. Try to load existing chain from disk
        if let Ok(data) = fs::read_to_string(CHAIN_FILE) {
            if let Ok(mut loaded_chain) = serde_json::from_str::<Blockchain>(&data) {
                // Re-initialize the parts that weren't saved
                loaded_chain.wallets = WalletManager::new();
                loaded_chain.marketplace = Marketplace::new();
                println!("📂 Loaded blockchain from '{}'", CHAIN_FILE);
                return loaded_chain;
            }
        }

        // 2. If no file, start fresh (Genesis)
        let genesis_block = Block::new(0, vec![], "0".to_string());
        
        let mut authorized_devices = HashSet::new();
        authorized_devices.insert("yuki-iot-alpha-001".to_string());
        authorized_devices.insert("yuki-iot-beta-999".to_string());
        
        Self {
            chain: vec![genesis_block],
            wallets: WalletManager::new(),
            marketplace: Marketplace::new(),
            stake_amount: 5,
            tasks_for_validation: Vec::new(),
            tasks_for_mining: Vec::new(),
            authorized_devices,
            used_evidence_urls: HashSet::new(),
        }
    }

    pub fn save_chain(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write(CHAIN_FILE, data);
        }
    }

    // --- REWARDS & VALIDATION LOGIC ---

    fn calculate_reward(&self, metadata_json: &str) -> u64 {
        let v: Value = match serde_json::from_str(metadata_json) {
            Ok(val) => val,
            Err(_) => return 1,
        };

        let reward = match v["type"].as_str() {
            Some("tree_planting") => {
                let count = v["count"].as_u64().unwrap_or(1);
                count * 1 
            },
            Some("plastic_recycling") => {
                let weight = v["weight_kg"].as_f64().unwrap_or(1.0);
                (weight * 0.5) as u64
            },
            Some("aqi_data") => 5,
            _ => 1,
        };
        if reward > 1000 { 1000 } else { reward }
    }

    fn run_decentralized_ai_check(&self, image_url: &str, task_type: &str) -> (f64, String) {
        if image_url.contains("fake") || image_url.contains("stock") || image_url.is_empty() {
            return (0.01, "flagged_keyword".to_string());
        }
        // Deterministic hash-based scoring
        let input = format!("{}{}", image_url, task_type);
        let hash = hash_data(&input);
        let score_byte = u8::from_str_radix(&hash[0..2], 16).unwrap_or(0);
        let confidence = score_byte as f64 / 255.0;
        let label = if confidence > 0.8 { "verified_match" } else if confidence > 0.5 { "uncertain" } else { "no_match_detected" };
        (confidence, label.to_string())
    }

    fn run_smart_contract_validation(&self, metadata: &Value) -> (bool, String) {
        // 1. Anti-Replay
        if let Some(url) = metadata["evidence"].as_str() {
            if self.used_evidence_urls.contains(url) {
                return (false, "⚠️ REPLAY ATTACK: Evidence already used.".to_string());
            }
        }
        // 2. IoT Signature
        if let Some(task_type) = metadata["type"].as_str() {
            if task_type == "aqi_data" {
                if let Some(dev_id) = metadata["device_id"].as_str() {
                    if !self.authorized_devices.contains(dev_id) {
                         return (false, format!("⚠️ UNAUTHORIZED DEVICE: '{}'", dev_id));
                    }
                } else { return (false, "⚠️ INVALID PACKET: Missing Device ID".to_string()); }
            }
        }
        // 3. AI Check
        if let Some(task_type) = metadata["type"].as_str() {
            if task_type == "tree_planting" || task_type == "plastic_recycling" {
                if let Some(url) = metadata["evidence"].as_str() {
                    let (confidence, label) = self.run_decentralized_ai_check(url, task_type);
                    if confidence < 0.80 {
                        return (false, format!("⚠️ AI REJECTION: {:.1}% ({})", confidence * 100.0, label));
                    }
                }
            }
        }
        (true, "✅ Validation Passed.".to_string())
    }

    pub fn submit_task(&mut self, wallet_address: &str, task: String, proof_metadata: String) -> Option<Transaction> {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < self.stake_amount {
                println!("❌ Stake failed. Insufficient balance.");
                return None;
            }
            wallet.balance_yuki -= self.stake_amount;
            let calculated_reward = self.calculate_reward(&proof_metadata);
            let transaction = Transaction::new(wallet_address.to_string(), "System-Reward-Pool".to_string(), calculated_reward, task, proof_metadata);
            
            self.tasks_for_validation.push(transaction.clone());
            self.save_chain(); // SAVE STATE
            self.wallets.save_wallets();
            Some(transaction)
        } else {
            None
        }
    }

    pub fn run_automated_validation(&mut self) -> Vec<(String, TaskStatus)> {
        let mut results = Vec::new();
        for i in (0..self.tasks_for_validation.len()).rev() {
            let task = self.tasks_for_validation[i].clone();
            let mut is_valid = false;
            let mut reason = "Unknown".to_string();

            if let Ok(v) = serde_json::from_str::<Value>(&task.proof_metadata) {
                let (passed, msg) = self.run_smart_contract_validation(&v);
                is_valid = passed;
                reason = msg;
                if passed { if let Some(url) = v["evidence"].as_str() { self.used_evidence_urls.insert(url.to_string()); } }
            }
            
            if is_valid {
                println!("\n[AUTO-VALIDATOR] Task {} APPROVED: {}", task.task, reason);
                let mut validated_task = self.tasks_for_validation.remove(i);
                validated_task.status = TaskStatus::Validated;
                self.tasks_for_mining.push(validated_task);
                results.push((task.task, TaskStatus::Validated));
            } else {
                println!("\n[AUTO-VALIDATOR] Task {} REJECTED: {}", task.task, reason);
                let _ = self.tasks_for_validation.remove(i);
                results.push((task.task, TaskStatus::Rejected));
                self.wallets.save_wallets(); // Burn stake
            }
        }
        self.save_chain(); // SAVE STATE
        results
    }

    pub fn mine_block(&mut self) -> Option<Block> {
        if self.tasks_for_mining.is_empty() { return None; }
        let mut transactions_for_block = Vec::new();
        while let Some(mut task) = self.tasks_for_mining.pop() {
            if let Some(wallet) = self.wallets.get_mut_wallet(&task.sender) {
                wallet.balance_yuki += task.amount + self.stake_amount;
                task.receiver = task.sender.clone();
                transactions_for_block.push(task);
            }
        }
        if transactions_for_block.is_empty() { return None; }
        let previous_block = self.chain.last().unwrap();
        let new_block = Block::new(previous_block.index + 1, transactions_for_block, previous_block.hash.clone());
        println!("✅ New block {} mined!", new_block.hash);
        self.chain.push(new_block.clone());
        
        self.save_chain(); // SAVE STATE
        self.wallets.save_wallets();
        Some(new_block)
    }

    // Helper functions needed by API and P2P
    pub fn create_wallet(&mut self) -> Wallet {
        let w = self.wallets.create_wallet();
        self.wallets.save_wallets();
        w
    }
    pub fn view_wallets(&self) { self.wallets.view_wallets(); }
    pub fn marketplace_menu(&mut self) { self.marketplace.menu(&mut self.wallets); }
    
    pub fn add_block_from_network(&mut self, block: Block) {
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
    pub fn add_task_from_network(&mut self, tx: Transaction) { 
        if !self.tasks_for_validation.iter().any(|t| t.task == tx.task) { self.tasks_for_validation.push(tx); } 
    }
    pub fn update_task_status_from_network(&mut self, task_id: &str, status: TaskStatus) {
         if let Some(pos) = self.tasks_for_validation.iter().position(|t| t.task == task_id) {
            match status {
                TaskStatus::Validated => { let t = self.tasks_for_validation.remove(pos); self.tasks_for_mining.push(t); }
                TaskStatus::Rejected => { self.tasks_for_validation.remove(pos); }
                _ => {}
            }
         }
    }
    pub fn convert_yuki_to_yg(&mut self, wallet_address: &str, amount: u64) {
         if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki >= amount { wallet.balance_yuki -= amount; wallet.balance_yg += amount; self.wallets.save_wallets(); }
         }
    }
    pub fn convert_yuki_to_yt(&mut self, wallet_address: &str, amount: u64) {
         if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki >= amount { wallet.balance_yuki -= amount; wallet.balance_yt += amount; self.wallets.save_wallets(); }
         }
    }
}