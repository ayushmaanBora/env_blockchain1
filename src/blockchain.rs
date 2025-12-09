use crate::transaction::{Transaction, TaskStatus};
use crate::wallet::{Wallet, WalletManager};
use crate::marketplace::Marketplace;
use crate::utils::hash_data;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashSet;
use std::io;

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
        Self {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    Block(Block),
    Transaction(Transaction),
    ValidationResult(String, TaskStatus),
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub wallets: WalletManager,
    pub marketplace: Marketplace,
    pub stake_amount: u64,
    pub tasks_for_validation: Vec<Transaction>,
    pub tasks_for_mining: Vec<Transaction>,
    // --- NEW: TRUST INFRASTRUCTURE ---
    pub authorized_devices: HashSet<String>, // The "Registry" of allowed IoT IDs
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, vec![], "0".to_string());
        let wallets = WalletManager::new();
        
        // Seed the registry with some "known good" device IDs (Public Keys)
        let mut authorized_devices = HashSet::new();
        authorized_devices.insert("yuki-iot-alpha-001".to_string());
        authorized_devices.insert("yuki-iot-beta-999".to_string());
        // In a real app, this would be updated via Governance votes

        Self {
            chain: vec![genesis_block],
            wallets,
            marketplace: Marketplace::new(),
            stake_amount: 5,
            tasks_for_validation: Vec::new(),
            tasks_for_mining: Vec::new(),
            authorized_devices,
        }
    }

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

    // --- AI & IOT VALIDATION LAYER ---

    // Simulation of an AI Computer Vision Model
    // Inputs: The evidence URL. 
    // Outputs: A confidence score (0.0 to 1.0) and a detected label.
    fn run_ai_inference(&self, image_url: &str, expected_type: &str) -> (f64, String) {
        println!("   🤖 [AI Model] Analyzing image: {} ...", image_url);
        
        // TODO: Connect this to a Python microservice or ONNX runtime
        // For simulation, we'll check if the URL contains "fake" to simulate a fail.
        if image_url.contains("fake") || image_url.contains("stock") {
            return (0.05, "stock_photo_detected".to_string());
        }

        // Simulate success for valid-looking inputs
        match expected_type {
            "tree_planting" => (0.98, "sapling_planted".to_string()),
            "plastic_recycling" => (0.92, "plastic_waste".to_string()),
            _ => (0.5, "unknown_object".to_string()),
        }
    }

    // Simulation of Cryptographic Signature Verification
    fn verify_iot_device(&self, device_id: &str) -> bool {
        // Check if the device is in our "Trusted Registry"
        self.authorized_devices.contains(device_id)
    }

    // --- UPDATED SMART CHECK ---
    fn run_smart_security_check(&self, metadata: &Value) -> (bool, String) {
        
        // 1. IoT Device Verification (The "Authorized Device ID" check)
        if let Some(task_type) = metadata["type"].as_str() {
            if task_type == "aqi_data" {
                if let Some(dev_id) = metadata["device_id"].as_str() {
                    if !self.verify_iot_device(dev_id) {
                         return (false, format!("⚠️  UNAUTHORIZED DEVICE: ID '{}' is not in the trusted registry.", dev_id));
                    }
                } else {
                    return (false, "⚠️  MISSING ID: AQI tasks must include a Device ID.".to_string());
                }
            }
        }

        // 2. AI Computer Vision Check
        if let Some(task_type) = metadata["type"].as_str() {
            if task_type == "tree_planting" || task_type == "plastic_recycling" {
                if let Some(url) = metadata["evidence"].as_str() {
                    let (confidence, label) = self.run_ai_inference(url, task_type);
                    
                    println!("   🤖 [AI Result] Label: '{}', Confidence: {:.2}%", label, confidence * 100.0);

                    if confidence < 0.70 {
                        return (false, format!("⚠️  AI REJECTION: Image does not appear to match task (Confidence: {:.1}%).", confidence * 100.0));
                    }
                } else {
                    return (false, "⚠️  MISSING PROOF: Photo evidence URL is required.".to_string());
                }
            }
        }

        // 3. Anomaly Detection (Logic Check)
        if let Some(count) = metadata["count"].as_u64() {
            if count > 500 {
                return (false, "⚠️  ANOMALY: Claiming >500 trees in one task is statistically improbable.".to_string());
            }
        }

        (true, "✅ Automated Checks Passed.".to_string())
    }

    pub fn submit_task(&mut self, wallet_address: &str, task: String, proof_metadata: String) -> Option<Transaction> {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < self.stake_amount {
                println!("❌ Stake failed. Insufficient balance.");
                return None;
            }
            wallet.balance_yuki -= self.stake_amount;

            let calculated_reward = self.calculate_reward(&proof_metadata);

            let transaction = Transaction::new(
                wallet_address.to_string(),
                "System-Reward-Pool".to_string(),
                calculated_reward,
                task,
                proof_metadata,
            );
            
            self.tasks_for_validation.push(transaction.clone());

            println!("✅ Task submitted! Potential Reward: {} Yuki. Stake held.", calculated_reward);
            self.wallets.save_wallets();
            Some(transaction)
        } else {
            println!("❌ Wallet not found.");
            None
        }
    }
    
    pub fn validate_pending_tasks(&mut self) -> Vec<(String, TaskStatus)> {
        let mut results = Vec::new();
        if self.tasks_for_validation.is_empty() {
            println!("No tasks are waiting for validation.");
            return results;
        }

        println!("\n--- 🕵️ SMART VALIDATION CONSOLE ---");
        for i in (0..self.tasks_for_validation.len()).rev() {
            let task = &self.tasks_for_validation[i];
            
            println!("\n--------------------------------");
            println!("Task ID: {}", task.task);
            println!("User:    {}", task.sender);
            println!("Claim:   {} Yuki", task.amount);
            
            let mut auto_check_passed = true;
            let mut auto_check_msg = String::new();

            if let Ok(v) = serde_json::from_str::<Value>(&task.proof_metadata) {
                // RUN THE AUTOMATED SECURITY CHECK
                let (passed, msg) = self.run_smart_security_check(&v);
                auto_check_passed = passed;
                auto_check_msg = msg;

                // Display data...
                if let Some(t) = v["type"].as_str() { println!("   Type:   {}", t); }
                if let Some(c) = v["count"].as_u64() { println!("   Count:  {}", c); }
                if let Some(d) = v["device_id"].as_str() { println!("   Device: {}", d); }
                if let Some(l) = v["location"].as_str() { println!("   GPS:    {}", l); }
            }
            
            println!("\n🔍 SYSTEM REPORT: {}", auto_check_msg);
            println!("--------------------------------");
            
            if !auto_check_passed {
                println!("⚠️  RECOMMENDATION: REJECT (AI/Security Flagged)");
            } else {
                println!("✅ RECOMMENDATION: APPROVE");
            }

            println!("Action: (A)pprove / (R)eject / (S)kip");

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            
            match choice.trim().to_lowercase().as_str() {
                "a" | "approve" => {
                    let mut validated_task = self.tasks_for_validation.remove(i);
                    validated_task.status = TaskStatus::Validated;
                    println!("✅ APPROVED. User rewarded.");
                    results.push((validated_task.task.clone(), validated_task.status.clone()));
                    self.tasks_for_mining.push(validated_task);
                }
                "r" | "reject" => {
                    let rejected_task = self.tasks_for_validation.remove(i);
                    println!("❌ REJECTED. Stake forfeited.");
                    results.push((rejected_task.task.clone(), TaskStatus::Rejected));
                    self.wallets.save_wallets();
                }
                _ => println!("Skipped."),
            }
        }
        results
    }

    pub fn mine_block(&mut self) -> Option<Block> {
        if self.tasks_for_mining.is_empty() {
            println!("No validated tasks to mine.");
            return None;
        }

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
        let new_block = Block::new(
            previous_block.index + 1,
            transactions_for_block,
            previous_block.hash.clone(),
        );

        println!("✅ New block {} mined!", new_block.hash);
        self.chain.push(new_block.clone());
        self.wallets.save_wallets();
        Some(new_block)
    }

    pub fn add_block_from_network(&mut self, block: Block) {
        let previous_block = self.chain.last().unwrap();
        if block.previous_hash == previous_block.hash {
            println!("Received valid block {} from network. Adding to chain.", block.hash);
            for tx in &block.transactions {
                self.tasks_for_mining.retain(|t| t.task != tx.task);
                self.tasks_for_validation.retain(|t| t.task != tx.task);
            }
            self.chain.push(block);
            self.wallets.save_wallets(); 
        }
    }

    pub fn add_task_from_network(&mut self, tx: Transaction) {
        if !self.tasks_for_validation.iter().any(|t| t.task == tx.task) &&
           !self.tasks_for_mining.iter().any(|t| t.task == tx.task) {
            println!("Received new task from network. Adding to validation pool.");
            self.tasks_for_validation.push(tx);
        }
    }

    pub fn update_task_status_from_network(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(pos) = self.tasks_for_validation.iter().position(|t| t.task == task_id) {
            match status {
                TaskStatus::Validated => {
                    let task = self.tasks_for_validation.remove(pos);
                    self.tasks_for_mining.push(task);
                }
                TaskStatus::Rejected => {
                    self.tasks_for_validation.remove(pos);
                }
                _ => {}
            }
        }
    }

    pub fn convert_yuki_to_yg(&mut self, wallet_address: &str, amount: u64) {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < amount {
                println!("❌ Insufficient Yuki balance.");
                return;
            }
            wallet.balance_yuki -= amount;
            wallet.balance_yg += amount;
            self.wallets.save_wallets();
            println!("✅ Converted {} Yuki to YG.", amount);
        }
    }

    pub fn convert_yuki_to_yt(&mut self, wallet_address: &str, amount: u64) {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < amount {
                println!("❌ Insufficient Yuki balance.");
                return;
            }
            wallet.balance_yuki -= amount;
            wallet.balance_yt += amount;
            self.wallets.save_wallets();
            println!("✅ Converted {} Yuki to YT.", amount);
        }
    }

    pub fn create_wallet(&mut self) -> Wallet {
        let wallet = self.wallets.create_wallet();
        self.wallets.save_wallets();
        wallet
    }

    pub fn view_wallets(&self) {
        self.wallets.view_wallets();
    }

    pub fn marketplace_menu(&mut self) {
        self.marketplace.menu(&mut self.wallets);
    }
}