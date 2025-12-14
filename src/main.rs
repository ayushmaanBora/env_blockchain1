mod blockchain;
mod wallet;
mod p2p;
mod marketplace; // We can keep this for viewing, but trading is disabled in logic
mod transaction;
mod utils;
mod api; 

use blockchain::{Blockchain, NetworkMessage};
use p2p::{P2PEvent, YUKI_TOPIC};
use std::error::Error;
use libp2p::{
    gossipsub::{Event as GossipsubEvent, IdentTopic},
    swarm::SwarmEvent,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::select;
use futures::StreamExt;
use serde_json::json; 
use std::sync::{Arc, Mutex}; 
use rand::{distributions::Alphanumeric, Rng}; // For simulating signatures

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🏭 Yuki Industrial Protocol v1.0 Starting...");
    
    // Initialize
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mut swarm = p2p::build_swarm()?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
    let blockchain_api = blockchain.clone();
    tokio::spawn(async move { api::start_api_server(blockchain_api).await; });

    println!("🌍 Industrial Sentinel Active. API: http://localhost:3030");

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    loop {
        println!("\n🏭 INDUSTRIAL COMMAND CONSOLE:");
        println!("1.  Submit Restoration Proof (EARN)");
        println!("2.  Request Emission Permit (BURN)");
        println!("3.  View Corporate Wallets");
        println!("4.  View Ledger");
        println!("5.  Sentinel Status (Peers)");
        println!("6.  Register New Wallet");
        println!("7.  Run Automated Compliance Check");
        println!("8.  Sync & Mine Block");
        println!("9.  Exit");

        select! {
            line = stdin.next_line() => {
                let choice = match line {
                    Ok(Some(line_str)) => line_str,
                    Ok(None) => "9".to_string(),
                    Err(_) => "9".to_string(),
                };

                match choice.trim() {
                    "1" => {
                        // --- INDUSTRIAL TASK SUBMISSION ---
                        println!("Enter Corporate Wallet Address:");
                        let mut wallet = String::new(); std::io::stdin().read_line(&mut wallet)?;

                        println!("\nSelect Restoration Operation:");
                        println!("1. Carbon Capture (Direct Air Capture)");
                        println!("2. Industrial Wastewater Treatment");
                        let mut type_choice = String::new(); std::io::stdin().read_line(&mut type_choice)?;

                        // Simulate Hardware Data
                        let sentinel_id = "yuki-industrial-01"; 
                        // Generate a random "signature" to simulate the Secure Element
                        let signature: String = rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(16)
                            .map(char::from)
                            .collect();

                        let (task_type, metadata) = match type_choice.trim() {
                            "1" => {
                                println!("Tons of CO2 Captured?");
                                let mut tons = String::new(); std::io::stdin().read_line(&mut tons)?;
                                ("carbon_capture", json!({
                                    "type": "carbon_capture",
                                    "sentinel_id": sentinel_id,
                                    "tons_captured": tons.trim().parse::<f64>().unwrap_or(0.0),
                                    "hardware_signature": signature 
                                }))
                            },
                            "2" => {
                                println!("Liters of Water Treated?");
                                let mut lit = String::new(); std::io::stdin().read_line(&mut lit)?;
                                ("wastewater_treatment", json!({
                                    "type": "wastewater_treatment",
                                    "sentinel_id": sentinel_id,
                                    "liters_treated": lit.trim().parse::<u64>().unwrap_or(0),
                                    "hardware_signature": signature
                                }))
                            },
                            _ => ("unknown", json!({}))
                        };

                        if task_type != "unknown" {
                            let task_name = format!("{}-{}", task_type, chrono::Utc::now().timestamp());
                            // LOCK & SUBMIT
                            if let Some(tx) = blockchain.lock().unwrap().submit_industrial_task(wallet.trim(), task_name, metadata.to_string()) {
                                let msg = NetworkMessage::Transaction(tx);
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let _ = swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), json.as_bytes());
                                }
                            }
                        }
                    }
                    "2" => {
                        // --- BURN TO EMIT ---
                        println!("Enter Corporate Wallet Address:");
                        let mut w = String::new(); std::io::stdin().read_line(&mut w)?;
                        println!("📉 EMISSION REQUEST: How many tons of CO2 to emit?");
                        let mut t = String::new(); std::io::stdin().read_line(&mut t)?;
                        
                        // LOCK & BURN
                        blockchain.lock().unwrap().request_emission_permit(w.trim(), t.trim().parse().unwrap_or(0));
                    }
                    "3" => blockchain.lock().unwrap().view_wallets(),
                    "4" => blockchain.lock().unwrap().chain.iter().for_each(|block| println!("{:#?}", block)),
                    "5" => {
                        println!("Connected Sentinels:");
                        for peer in swarm.behaviour().mdns.discovered_nodes() { println!("{}", peer); }
                    },
                    "6" => {
                        let w = blockchain.lock().unwrap().create_wallet();
                        println!("✅ New Corporate Wallet Registered: {}", w.address);
                    }
                    "7" => {
                        let results = blockchain.lock().unwrap().run_automated_validation();
                        for (task_id, status) in results {
                            let msg = NetworkMessage::ValidationResult(task_id, status);
                            let _ = swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), serde_json::to_string(&msg).unwrap().as_bytes());
                        }
                    }
                    "8" => {
                        if let Some(block) = blockchain.lock().unwrap().mine_block() {
                            let msg = NetworkMessage::Block(block);
                            let _ = swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(YUKI_TOPIC), serde_json::to_string(&msg).unwrap().as_bytes());
                        }
                    }
                    "9" | "exit" => break,
                    _ => println!("❌ Invalid Command."),
                }
            },
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(P2PEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                        if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match msg {
                                NetworkMessage::Block(b) => { println!("\n[NET] Ledger Update."); blockchain.lock().unwrap().add_block_from_network(b); },
                                NetworkMessage::Transaction(t) => { println!("\n[NET] Incoming Telemetry."); blockchain.lock().unwrap().add_task_from_network(t); },
                                NetworkMessage::ValidationResult(id, s) => { println!("\n[NET] Compliance Update."); blockchain.lock().unwrap().update_task_status_from_network(&id, s); }
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