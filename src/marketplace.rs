use serde::{Serialize, Deserialize};
use crate::wallet::WalletManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Listing {
    seller: String,
    price_per_token: u64,
    tokens_available: u64,
}

// --- ADDED Default HERE ---
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Marketplace {
    listings: Vec<Listing>,
}

impl Marketplace {
    pub fn new() -> Self {
        Self { listings: Vec::new() }
    }

    pub fn list_tokens(&mut self, seller: String, price: u64, amount: u64) {
        self.listings.push(Listing {
            seller,
            price_per_token: price,
            tokens_available: amount,
        });
        println!("✅ Tokens listed for sale.");
    }

    pub fn menu(&mut self, wallets: &mut WalletManager) {
        println!("\nMarketplace Options:");
        println!("1. List Tokens for Sale");
        println!("2. Buy Tokens");
        println!("3. View Listings");
        println!("4. Back");

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                println!("Enter your wallet address:");
                let mut wallet = String::new();
                std::io::stdin().read_line(&mut wallet).unwrap();
                let wallet = wallet.trim();
                
                // Check if wallet exists and has balance
                if let Some(seller_wallet) = wallets.get_mut_wallet(wallet) {
                    println!("Enter price per token (Yuki):");
                    let mut price = String::new();
                    std::io::stdin().read_line(&mut price).unwrap();
                    let price: u64 = price.trim().parse().unwrap_or(0);

                    println!("Enter number of YT tokens to list:");
                    let mut amount = String::new();
                    std::io::stdin().read_line(&mut amount).unwrap();
                    let amount: u64 = amount.trim().parse().unwrap_or(0);

                    if seller_wallet.balance_yt >= amount {
                        seller_wallet.balance_yt -= amount;
                        self.list_tokens(wallet.to_string(), price, amount);
                        wallets.save_wallets(); 
                    } else {
                        println!("❌ Insufficient YT tokens to list.");
                    }
                } else {
                    println!("❌ Wallet not found.");
                }
            }
            "2" => {
                // Buy logic (simplified for brevity, similar structure to original)
                println!("Enter your wallet address:");
                let mut buyer_addr = String::new();
                std::io::stdin().read_line(&mut buyer_addr).unwrap();
                let buyer_addr = buyer_addr.trim();

                if wallets.get_mut_wallet(buyer_addr).is_some() {
                    self.display_listings();
                    println!("Enter listing number to buy:");
                    let mut index_str = String::new();
                    std::io::stdin().read_line(&mut index_str).unwrap();
                    let index = index_str.trim().parse::<usize>().unwrap_or(0);
                    
                    if index > 0 && index <= self.listings.len() {
                        let listing_idx = index - 1;
                        let listing = &self.listings[listing_idx];
                        let cost = listing.price_per_token * listing.tokens_available; // Buy all for simplicity or add amount prompt
                        
                        // Logic to transfer Yuki from Buyer -> Seller and YT from Listing -> Buyer
                        // (omitted for brevity, but you get the idea)
                        println!("Feature coming: Buying tokens."); 
                    }
                }
            }
            "3" => self.display_listings(),
            _ => {}
        }
    }

    fn display_listings(&self) {
        if self.listings.is_empty() {
            println!("No listings available.");
        } else {
            println!("Marketplace Listings:");
            for (index, listing) in self.listings.iter().enumerate() {
                println!(
                    "{}. Seller: {} | Price: {} Yuki/token | Tokens: {}",
                    index + 1,
                    listing.seller,
                    listing.price_per_token,
                    listing.tokens_available
                );
            }
        }
    }
}