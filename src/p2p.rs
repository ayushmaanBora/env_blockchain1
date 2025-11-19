use libp2p::{
    // core::upgrade, // Removed (unused)
    gossipsub::{
        // self, // Removed (unused)
        Behaviour as Gossipsub, ConfigBuilder as GossipsubConfigBuilder,
        Event as GossipsubEvent, IdentTopic, MessageAuthenticity,
    },
    identity,
    mdns::{self, Config as MdnsConfig, Event as MdnsEvent},
    noise,
    swarm::{NetworkBehaviour, Swarm},
    tcp, // We only need the module, not the Config alias
    yamux::Config as YamuxConfig, // We need this alias
    // dns::tokio::Transport as DnsTransport, // Removed (unused)
    PeerId, SwarmBuilder,
    // Transport, // Removed (unused)
};
use std::error::Error;

// Define the topic
pub const YUKI_TOPIC: &str = "yuki";

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "P2PEvent")]
pub struct P2PNetwork {
    pub gossipsub: Gossipsub,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum P2PEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
}

impl From<GossipsubEvent> for P2PEvent {
    fn from(event: GossipsubEvent) -> Self {
        P2PEvent::Gossipsub(event)
    }
}

impl From<MdnsEvent> for P2PEvent {
    fn from(event: MdnsEvent) -> Self {
        P2PEvent::Mdns(event)
    }
}

pub fn build_swarm() -> Result<Swarm<P2PNetwork>, Box<dyn Error>> {
    // Generate identity
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local PeerId: {}", peer_id);

    // --- BEHAVIOUR ---
    let gossipsub_config = GossipsubConfigBuilder::default()
        .build()
        .expect("Valid gossipsub config");

    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )?;

    gossipsub.subscribe(&IdentTopic::new(YUKI_TOPIC))?;
    let mdns = mdns::tokio::Behaviour::new(MdnsConfig::default(), peer_id)?;
    let behaviour = P2PNetwork { gossipsub, mdns };

    // --- SWARM ---
    let swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            YamuxConfig::default, // <-- THE FIX: Use the alias 'YamuxConfig'
        )?
        .with_dns()?
        .with_behaviour(|_key| behaviour)?
        .build();

    Ok(swarm)
}