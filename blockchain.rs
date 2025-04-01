use crate::transaction::Transaction;
use crate::wallet::{Wallet, WalletManager};
use crate::marketplace::Marketplace;
use crate::utils::hash_data;
use chrono::Utc;
use serde::{Serialize, Deserialize};

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

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub wallets: WalletManager,
    pub marketplace: Marketplace,
    pub mining_reward: u64,
    pub stake_amount: u64,
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
        }
    }

    pub fn add_block(&mut self, wallet_address: &str, task: String, proof_metadata: String) {
        if let Some(wallet) = self.wallets.get_mut_wallet(wallet_address) {
            if wallet.balance_yuki < self.stake_amount {
                println!("❌ Stake failed. Insufficient balance.");
                return;
            }

            wallet.balance_yuki -= self.stake_amount;
            wallet.balance_yuki += self.mining_reward;

            let transaction = Transaction {
                sender: "System".to_string(),
                receiver: wallet_address.to_string(),
                amount: self.mining_reward,
                task,
                proof_metadata,
                verified: true,
            };

            let previous_block = self.chain.last().unwrap();
            let new_block = Block::new(
                previous_block.index + 1,
                vec![transaction],
                previous_block.hash.clone(),
            );
            self.chain.push(new_block);

            println!("✅ Task verified! Block added. Tokens awarded.");
            self.wallets.save_wallets();
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
