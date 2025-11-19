use serde::{Serialize, Deserialize};
use crate::wallet::{WalletManager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Listing {
    pub seller: String,
    pub price_per_token: u64,
    pub tokens_available: u64,
}

#[derive(Debug, Serialize, Deserialize)]
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
        loop {
            println!("\nMarketplace Options:");
            println!("1. List Tokens for Sale");
            println!("2. Buy Tokens");
            println!("3. View Listings");
            println!("4. Exit Marketplace");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).unwrap();

            match choice.trim() {
                "1" => self.list_tokens_menu(wallets),
                "2" => self.buy_tokens_menu(wallets),
                "3" => self.display_listings(),
                "4" => break,
                _ => println!("❌ Invalid choice. Try again."),
            }
        }
    }

    fn list_tokens_menu(&mut self, wallets: &mut WalletManager) {
        println!("Enter your wallet address:");
        let mut wallet = String::new();
        std::io::stdin().read_line(&mut wallet).unwrap();
        let wallet = wallet.trim();

        if let Some(seller_wallet) = wallets.get_mut_wallet(wallet) {
            println!("Enter price per token:");
            let mut price = String::new();
            std::io::stdin().read_line(&mut price).unwrap();
            let price: u64 = price.trim().parse().unwrap_or(0);

            println!("Enter number of tokens to list:");
            let mut amount = String::new();
            std::io::stdin().read_line(&mut amount).unwrap();
            let amount: u64 = amount.trim().parse().unwrap_or(0);

            if seller_wallet.balance_yt >= amount {
                seller_wallet.balance_yt -= amount;
                self.list_tokens(wallet.to_string(), price, amount);
            } else {
                println!("❌ Insufficient tokens to list.");
            }
        } else {
            println!("❌ Wallet not found.");
        }
    }

    fn buy_tokens_menu(&mut self, wallets: &mut WalletManager) {
        println!("Enter your wallet address:");
        let mut buyer_wallet_addr = String::new();
        std::io::stdin().read_line(&mut buyer_wallet_addr).unwrap();
        let buyer_wallet_addr = buyer_wallet_addr.trim();

        if wallets.get_mut_wallet(buyer_wallet_addr).is_some() {
            self.display_listings();

            println!("Enter listing number to buy:");
            let mut listing_index = String::new();
            std::io::stdin().read_line(&mut listing_index).unwrap();
            let listing_index: usize = listing_index.trim().parse().unwrap_or(0) - 1;

            if listing_index < self.listings.len() {
                let listing = self.listings[listing_index].clone();
                let seller_id = listing.seller.clone();
                let tokens_available = listing.tokens_available;
                let price_per_token = listing.price_per_token;

                println!("Enter amount of tokens to buy (available: {}):", tokens_available);
                let mut amount = String::new();
                std::io::stdin().read_line(&mut amount).unwrap();
                let amount: u64 = amount.trim().parse().unwrap_or(0);

                if amount <= tokens_available {
                    let total_price = price_per_token * amount;

                    if let Some(buyer_wallet) = wallets.get_mut_wallet(buyer_wallet_addr) {
                        if buyer_wallet.balance_yuki >= total_price {
                            buyer_wallet.balance_yuki -= total_price;
                            buyer_wallet.balance_yt += amount;

                            if let Some(seller_wallet) = wallets.get_mut_wallet(&seller_id) {
                                seller_wallet.balance_yuki += total_price;
                            }

                            self.listings[listing_index].tokens_available -= amount;

                            println!("✅ Tokens purchased successfully!");

                            if self.listings[listing_index].tokens_available == 0 {
                                self.listings.remove(listing_index);
                            }
                        } else {
                            println!("❌ Insufficient funds to buy tokens.");
                        }
                    }
                } else {
                    println!("❌ Not enough tokens available in the listing.");
                }
            } else {
                println!("❌ Invalid listing number.");
            }
        } else {
            println!("❌ Wallet not found.");
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
