mod blockchain;
mod wallet;
mod p2p;
mod marketplace;
mod transaction;
mod utils;

use blockchain::Blockchain;
use std::io;

fn main() {
    let mut blockchain = Blockchain::new();
    println!("🌱 Riti Blockchain Initialized!");

    loop {
        println!("\n🌍 Options:");
        println!("1. Submit Task");
        println!("2. Marketplace");
        println!("3. View Blockchain");
        println!("4. Create Wallet");
        println!("5. View Wallets");
        println!("6. Exit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                println!("Enter Wallet Address:");
                let mut wallet = String::new();
                io::stdin().read_line(&mut wallet).unwrap();

                println!("Enter Task Name:");
                let mut task = String::new();
                io::stdin().read_line(&mut task).unwrap();

                println!("Enter Proof Metadata:");
                let mut metadata = String::new();
                io::stdin().read_line(&mut metadata).unwrap();

                blockchain.add_block(wallet.trim(), task.trim().to_string(), metadata.trim().to_string());
            }
            "2" => blockchain.marketplace_menu(),
            "3" => blockchain.chain.iter().for_each(|block| println!("{:#?}", block)),
            "4" => {
                let wallet = blockchain.create_wallet();
                println!("✅ Wallet created! Address: {}", wallet.address);
            }
            "5" => blockchain.view_wallets(),
            "6" => {
                println!("Exiting...");
                break;
            }
            _ => println!("❌ Invalid choice. Try again!"),
        }
    }
}
