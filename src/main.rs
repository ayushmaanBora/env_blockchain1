mod blockchain;
mod wallet;
mod p2p;
mod marketplace;
mod transaction;
mod utils;

use blockchain::{Blockchain, NetworkMessage};
use p2p::{P2PEvent, YUKI_TOPIC};
use std::error::Error;
use libp2p::{
    gossipsub::{Event as GossipsubEvent, IdentTopic},
    mdns::Event as MdnsEvent,
    swarm::SwarmEvent,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::select;
use futures::StreamExt; // This gives us .select_next_some()

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut blockchain = Blockchain::new();
    println!("🌱 Riti Blockchain Initialized!");

    let mut swarm = p2p::build_swarm()?;

    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse()?)
        .unwrap();

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    loop {
        println!("\n🌍 Options:");
        println!("--- User ---");
        println!("1. Submit Task (Submit for validation)");
        println!("2. Marketplace");
        println!("3. View Wallets");
        println!("--- Network ---");
        println!("4. View Blockchain");
        println!("5. View Peers");
        println!("6. Create Wallet");
        println!("--- Validator/Miner ---");
        println!("7. Validate Pending Tasks (Admin)");
        println!("8. Mine Block (From validated tasks)");
        println!("--- Tokenomics ---");
        println!("9. Convert Yuki -> YukiGreen (YG)");
        println!("10. Convert Yuki -> YukiTrade (YT)");
        println!("11. Exit");

        select! {
            line = stdin.next_line() => {
                let choice = match line {
                    Ok(Some(line_str)) => line_str,
                    Ok(None) => "11".to_string(),
                    Err(_) => "11".to_string(),
                };

                match choice.trim() {
                    "1" => {
                        println!("Enter Wallet Address:");
                        let mut wallet = String::new();
                        std::io::stdin().read_line(&mut wallet)?;

                        println!("Enter Task Name (must be unique):");
                        let mut task = String::new();
                        std::io::stdin().read_line(&mut task)?;

                        println!("Enter Proof Metadata (e.g., URL or coordinates):");
                        let mut metadata = String::new();
                        std::io::stdin().read_line(&mut metadata)?;

                        if let Some(tx) = blockchain.submit_task(wallet.trim(), task.trim().to_string(), metadata.trim().to_string()) {
                            let msg = NetworkMessage::Transaction(tx);
                            let json = serde_json::to_string(&msg)?;
                            swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes())?;
                        }
                    }
                    "2" => blockchain.marketplace_menu(),
                    "3" => blockchain.view_wallets(),
                    "4.0" => blockchain.chain.iter().for_each(|block| println!("{:#?}", block)),
                    "5" => {
                        println!("Connected Peers:");
                        for peer in swarm.behaviour().mdns.discovered_nodes() {
                            println!("{}", peer);
                        }
                    }
                    "6" => {
                        let wallet = blockchain.create_wallet();
                        println!("✅ Wallet created! Address: {}", wallet.address);
                    }
                    // --- NEW VALIDATION OPTION ---
                    "7" => {
                        let results = blockchain.validate_pending_tasks();
                        // Broadcast validation results to the network
                        for (task_id, status) in results {
                            let msg = NetworkMessage::ValidationResult(task_id, status);
                            let json = serde_json::to_string(&msg)?;
                            swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes())?;
                        }
                    }
                    // --- UPDATED MINE OPTION ---
                    "8" => {
                        if let Some(block) = blockchain.mine_block() {
                            let msg = NetworkMessage::Block(block);
                            let json = serde_json::to_string(&msg)?;
                            swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes())?;
                        }
                    }
                    "9" => {
                        println!("Enter Wallet Address:");
                        let mut wallet = String::new();
                        std::io::stdin().read_line(&mut wallet)?;

                        println!("Enter amount of Yuki to convert to YG:");
                        let mut amount = String::new();
                        std::io::stdin().read_line(&mut amount)?;
                        let amount: u64 = amount.trim().parse().unwrap_or(0);
                        
                        blockchain.convert_yuki_to_yg(wallet.trim(), amount);
                    }
                    "10" => {
                        println!("Enter Wallet Address:");
                        let mut wallet = String::new();
                        std::io::stdin().read_line(&mut wallet)?;

                        println!("Enter amount of Yuki to convert to YT:");
                        let mut amount = String::new();
                        std::io::stdin().read_line(&mut amount)?;
                        let amount: u64 = amount.trim().parse().unwrap_or(0);

                        blockchain.convert_yuki_to_yt(wallet.trim(), amount);
                    }
                    "11" | "exit" => {
                        println!("Exiting...");
                        break;
                    }
                    _ => println!("❌ Invalid choice. Try again!"),
                }
            },

            // Handle network events
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(P2PEvent::Mdns(MdnsEvent::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("\n*** Discovered peer: {} ***", peer_id);
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(P2PEvent::Mdns(MdnsEvent::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("\n*** Lost peer: {} ***", peer_id);
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(P2PEvent::Gossipsub(GossipsubEvent::Message {
                        message, ..
                    })) => {
                        if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match msg {
                                NetworkMessage::Block(block) => {
                                    println!("\n*** Received new block from network. ***");
                                    blockchain.add_block_from_network(block);
                                }
                                NetworkMessage::Transaction(tx) => {
                                    println!("\n*** Received new task from network. ***");
                                    blockchain.add_task_from_network(tx);
                                }
                                NetworkMessage::ValidationResult(task_id, status) => {
                                    println!("\n*** Received validation result from network. ***");
                                    blockchain.update_task_status_from_network(&task_id, status);
                                }
                            }
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {}", address);
                    }
                    _ => {} 
                }
                // Reprint the menu after a network event
                println!("\n(Network event processed) 🌍 Options:");
                println!("1. Submit Task | 2. Marketplace | 3. View Wallets | 4. View Blockchain | 5. View Peers | 6. Create Wallet | 7. Validate Tasks | 8. Mine Block | 9. Convert YG | 10. Convert YT | 11. Exit");
            }
        }
    }

    Ok(())
}