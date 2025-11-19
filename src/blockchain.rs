use crate::transaction::{Transaction, TaskStatus};
use crate::wallet::{Wallet, WalletManager};
use crate::marketplace::Marketplace;
use crate::utils::hash_data;
use chrono::Utc;
use serde::{Serialize, Deserialize};
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
    ValidationResult(String, TaskStatus), // To sync validation results
}


pub struct Blockchain {
    pub chain: Vec<Block>,
    pub wallets: WalletManager,
    pub marketplace: Marketplace,
    pub mining_reward: u64,
    pub stake_amount: u64,
    // --- We now have two task pools ---
    pub tasks_for_validation: Vec<Transaction>,
    pub tasks_for_mining: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, vec![], "0".to_string());
        let wallets = WalletManager::new();
        Self {
            chain: vec![genesis_block],
            wallets,
            marketplace: Marketplace::new(),
            mining_reward: 10,
            stake_amount: 5,
            tasks_for_validation: Vec::new(),
            tasks_for_mining: Vec::new(),
        }
    }

    // A user submits a task. It goes into the validation pool.
    pub fn submit_task(&mut self, wallet_address: &str, task: String, proof_metadata: String) -> Option<Transaction> {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < self.stake_amount {
                println!("❌ Stake failed. Insufficient balance.");
                return None;
            }
            wallet.balance_yuki -= self.stake_amount; // Hold the stake

            let transaction = Transaction::new(
                wallet_address.to_string(),
                "System-Validation".to_string(),
                self.mining_reward,
                task,
                proof_metadata,
            );
            
            self.tasks_for_validation.push(transaction.clone());

            println!("✅ Task submitted for validation. Stake of {} held.", self.stake_amount);
            self.wallets.save_wallets();
            Some(transaction)
        } else {
            println!("❌ Wallet not found.");
            None
        }
    }
    
    // --- NEW VALIDATION FUNCTION ---
    pub fn validate_pending_tasks(&mut self) -> Vec<(String, TaskStatus)> {
        let mut results = Vec::new();
        if self.tasks_for_validation.is_empty() {
            println!("No tasks are waiting for validation.");
            return results;
        }

        println!("--- Pending Task Validation ---");
        // We iterate backwards so we can remove items without messing up the index
        for i in (0..self.tasks_for_validation.len()).rev() {
            let task = &self.tasks_for_validation[i];
            println!("\nValidating Task for: {}", task.sender);
            println!("  Task: {}", task.task);
            println!("  Proof: {}", task.proof_metadata);
            println!("Approve? (y/n/skip)");

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            
            match choice.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    let mut validated_task = self.tasks_for_validation.remove(i);
                    validated_task.status = TaskStatus::Validated;
                    println!("✅ Task Approved. Moved to mining pool.");
                    results.push((validated_task.task.clone(), validated_task.status.clone()));
                    self.tasks_for_mining.push(validated_task);
                }
                "n" | "no" => {
                    let rejected_task = self.tasks_for_validation.remove(i);
                    println!("❌ Task Rejected. Stake forfeited.");
                    // The stake is not returned, it's just... gone.
                    // In a real system, we'd send this to a community fund.
                    results.push((rejected_task.task.clone(), TaskStatus::Rejected));
                    self.wallets.save_wallets(); // Save the wallet state (with the stake removed)
                }
                _ => {
                    println!("-- Task Skipped. --");
                }
            }
        }
        results
    }

    // Mine Block now only uses the *validated* task pool
    pub fn mine_block(&mut self) -> Option<Block> {
        if self.tasks_for_mining.is_empty() {
            println!("No validated tasks to mine.");
            return None;
        }

        let mut transactions_for_block = Vec::new();
        
        // Take all tasks from the mining pool
        while let Some(mut task) = self.tasks_for_mining.pop() {
            if let Some(wallet) = self.wallets.get_mut_wallet(&task.sender) {
                // Give reward AND return stake
                wallet.balance_yuki += task.amount + self.stake_amount;
                task.receiver = task.sender.clone(); // Show who received the reward
                transactions_for_block.push(task);
            }
        }
        
        if transactions_for_block.is_empty() {
            println!("Failed to process tasks for mining.");
            return None;
        }

        println!("Mining new block with {} validated transactions...", transactions_for_block.len());

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
            // We must also update our local task pools
            for tx in &block.transactions {
                self.tasks_for_mining.retain(|t| t.task != tx.task);
                self.tasks_for_validation.retain(|t| t.task != tx.task);
            }
            self.chain.push(block);
            self.wallets.save_wallets(); 
        } else {
            println!("Received invalid block (hash mismatch). Discarding.");
        }
    }

    // Add a new task from the network to our validation pool
    pub fn add_task_from_network(&mut self, tx: Transaction) {
        // Check if we already have this task
        if !self.tasks_for_validation.iter().any(|t| t.task == tx.task) &&
           !self.tasks_for_mining.iter().any(|t| t.task == tx.task) {
            println!("Received new task from network. Adding to validation pool.");
            self.tasks_for_validation.push(tx);
        }
    }

    // Update local task pools based on validation results from the network
    pub fn update_task_status_from_network(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(pos) = self.tasks_for_validation.iter().position(|t| t.task == task_id) {
            match status {
                TaskStatus::Validated => {
                    let task = self.tasks_for_validation.remove(pos);
                    println!("Task {} validated by network. Moving to mining pool.", task_id);
                    self.tasks_for_mining.push(task);
                }
                TaskStatus::Rejected => {
                    self.tasks_for_validation.remove(pos);
                    println!("Task {} rejected by network. Removing.", task_id);
                }
                _ => {}
            }
        }
    }

    pub fn convert_yuki_to_yg(&mut self, wallet_address: &str, amount: u64) {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < amount {
                println!("❌ Conversion failed. Insufficient Yuki balance.");
                return;
            }
            wallet.balance_yuki -= amount;
            wallet.balance_yg += amount;
            self.wallets.save_wallets();
            println!("✅ Converted {} Yuki to {} YukiGreen (YG).", amount, amount);
        } else {
            println!("❌ Wallet not found.");
        }
    }

    pub fn convert_yuki_to_yt(&mut self, wallet_address: &str, amount: u64) {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < amount {
                println!("❌ Conversion failed. Insufficient Yuki balance.");
                return;
            }
            wallet.balance_yuki -= amount;
            wallet.balance_yt += amount;
            self.wallets.save_wallets();
            println!("✅ Converted {} Yuki to {} YukiTrade (YT).", amount, amount);
        } else {
            println!("❌ Wallet not found.");
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