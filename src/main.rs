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
use futures::StreamExt;
use serde_json::json; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut blockchain = Blockchain::new();
    println!("🌱 Riti Blockchain Initialized!");

    let mut swarm = p2p::build_swarm()?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    loop {
        println!("\n🌍 Options:");
        println!("1.  Submit Task (Guided Mode)");
        println!("2.  Marketplace");
        println!("3.  View Wallets");
        println!("4.  View Blockchain");
        println!("5.  View Peers");
        println!("6.  Create Wallet");
        println!("7.  Validate Pending Tasks (Admin)");
        println!("8.  Mine Block");
        println!("9.  Convert Yuki -> YG");
        println!("10. Convert Yuki -> YT");
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
                        // --- GUIDED TASK SUBMISSION ---
                        println!("Enter Wallet Address:");
                        let mut wallet = String::new();
                        std::io::stdin().read_line(&mut wallet)?;

                        println!("\nSelect Task Type:");
                        println!("1. Tree Planting (1 Yuki/tree)");
                        println!("2. Plastic Recycling (0.5 Yuki/kg)");
                        println!("3. AQI Data Collection (5 Yuki/report)");
                        let mut type_choice = String::new();
                        std::io::stdin().read_line(&mut type_choice)?;

                        let (task_type, metadata) = match type_choice.trim() {
                            "1" => {
                                println!("How many trees planted?");
                                let mut count = String::new();
                                std::io::stdin().read_line(&mut count)?;
                                let count: u64 = count.trim().parse().unwrap_or(0);
                                
                                println!("GPS Location (e.g. 12.34, 56.78):");
                                let mut loc = String::new();
                                std::io::stdin().read_line(&mut loc)?;
                                
                                println!("Photo Evidence URL:");
                                let mut url = String::new();
                                std::io::stdin().read_line(&mut url)?;

                                ("tree_planting", json!({
                                    "type": "tree_planting",
                                    "count": count,
                                    "location": loc.trim(),
                                    "evidence": url.trim()
                                }))
                            },
                            "2" => {
                                println!("Weight of plastic (kg)?");
                                let mut weight = String::new();
                                std::io::stdin().read_line(&mut weight)?;
                                let weight: f64 = weight.trim().parse().unwrap_or(0.0);
                                
                                println!("Recycling Center Location:");
                                let mut loc = String::new();
                                std::io::stdin().read_line(&mut loc)?;
                                
                                println!("Photo Evidence URL:");
                                let mut url = String::new();
                                std::io::stdin().read_line(&mut url)?;

                                ("plastic_recycling", json!({
                                    "type": "plastic_recycling",
                                    "weight_kg": weight,
                                    "location": loc.trim(),
                                    "evidence": url.trim()
                                }))
                            },
                            "3" => {
                                println!("Device ID (must start with 'yuki-iot-'):");
                                let mut dev = String::new();
                                std::io::stdin().read_line(&mut dev)?;
                                
                                println!("GPS Location:");
                                let mut loc = String::new();
                                std::io::stdin().read_line(&mut loc)?;
                                
                                println!("PM2.5 Value:");
                                let mut val = String::new();
                                std::io::stdin().read_line(&mut val)?;

                                ("aqi_data", json!({
                                    "type": "aqi_data",
                                    "device_id": dev.trim(),
                                    "location": loc.trim(),
                                    "pm25": val.trim()
                                }))
                            },
                            _ => ("unknown", json!({}))
                        };

                        if task_type != "unknown" {
                            let task_name = format!("{}-{}", task_type, chrono::Utc::now().timestamp());
                            let metadata_str = metadata.to_string();

                            if let Some(tx) = blockchain.submit_task(wallet.trim(), task_name, metadata_str) {
                                let msg = NetworkMessage::Transaction(tx);
                                let json = serde_json::to_string(&msg)?;
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes()) {
                                     println!("(Network Note: {:?})", e);
                                }
                            }
                        } else {
                            println!("❌ Invalid task type selected.");
                        }
                    }
                    "2" => blockchain.marketplace_menu(),
                    "3" => blockchain.view_wallets(),
                    "4" => blockchain.chain.iter().for_each(|block| println!("{:#?}", block)),
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
                    "7" => {
                        let results = blockchain.validate_pending_tasks();
                        for (task_id, status) in results {
                            let msg = NetworkMessage::ValidationResult(task_id, status);
                            let json = serde_json::to_string(&msg)?;
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes()) {
                                println!("(Network Note: {:?})", e);
                            }
                        }
                    }
                    "8" => {
                        if let Some(block) = blockchain.mine_block() {
                            let msg = NetworkMessage::Block(block);
                            let json = serde_json::to_string(&msg)?;
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes()) {
                                println!("(Network Note: {:?})", e);
                            }
                        }
                    }
                    "9" => {
                        println!("Enter Wallet Address:");
                        let mut w = String::new(); std::io::stdin().read_line(&mut w)?;
                        println!("Amount:");
                        let mut a = String::new(); std::io::stdin().read_line(&mut a)?;
                        blockchain.convert_yuki_to_yg(w.trim(), a.trim().parse().unwrap_or(0));
                    }
                    "10" => {
                        println!("Enter Wallet Address:");
                        let mut w = String::new(); std::io::stdin().read_line(&mut w)?;
                        println!("Amount:");
                        let mut a = String::new(); std::io::stdin().read_line(&mut a)?;
                        blockchain.convert_yuki_to_yt(w.trim(), a.trim().parse().unwrap_or(0));
                    }
                    "11" | "exit" => break,
                    _ => println!("❌ Invalid choice."),
                }
            },
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(P2PEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                        if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match msg {
                                NetworkMessage::Block(b) => { println!("\n[NET] New Block received."); blockchain.add_block_from_network(b); },
                                NetworkMessage::Transaction(t) => { println!("\n[NET] New Task received."); blockchain.add_task_from_network(t); },
                                NetworkMessage::ValidationResult(id, s) => { println!("\n[NET] Validation Update."); blockchain.update_task_status_from_network(&id, s); }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    Ok(())
}