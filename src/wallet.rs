use bip39::Mnemonic;
use rand::{RngCore, thread_rng};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;

const WALLET_FILE: &str = "wallets.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub address: String,
    pub balance_yuki: u64,
    pub balance_yg: u64,
    pub balance_yt: u64,
}

impl Wallet {
    pub fn new(address: String) -> Self {
        Self {
            address,
            balance_yuki: 10,
            balance_yg: 0,
            balance_yt: 0,
        }
    }
}

// --- ADDED #[derive(Default)] HERE ---
#[derive(Default)] 
pub struct WalletManager {
    wallets: HashMap<String, Wallet>,
}

impl WalletManager {
    pub fn new() -> Self {
        let wallets = Self::load_wallets();
        Self { wallets }
    }

    /// Create a new wallet from random 32 bytes => 24 words in bip39
    pub fn create_wallet(&mut self) -> Wallet {
        let mut rng = thread_rng();
        let mut entropy = [0u8; 32];
        rng.fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy).expect("Failed to create mnemonic");

        let words = mnemonic.to_string(); 
        println!("Mnemonic (save this!): {}", words);

        let address = crate::utils::hash_data(&format!("{:?}", entropy));
        let wallet = Wallet::new(address.clone());
        self.wallets.insert(address.clone(), wallet.clone());
        wallet
    }

    pub fn get_mut_wallet(&mut self, address: &str) -> Option<&mut Wallet> {
        self.wallets.get_mut(address)
    }

    // Helper for API
    pub fn get_all_wallets(&self) -> Vec<Wallet> {
        self.wallets.values().cloned().collect()
    }

    pub fn view_wallets(&self) {
        for wallet in self.wallets.values() {
            println!(
                "Address: {} | Yuki: {} | YG: {} | YT: {}",
                wallet.address, wallet.balance_yuki, wallet.balance_yg, wallet.balance_yt
            );
        }
    }

    pub fn save_wallets(&self) {
        let data = serde_json::to_string(&self.wallets).expect("Failed to serialize wallets.");
        fs::write(WALLET_FILE, data).expect("Failed to save wallets.");
    }

    pub fn load_wallets() -> HashMap<String, Wallet> {
        if let Ok(data) = fs::read_to_string(WALLET_FILE) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }
}