use libp2p::{
    core::upgrade,
    gossipsub::{
        self, Gossipsub, GossipsubConfigBuilder, GossipsubEvent, IdentTopic,
        MessageAuthenticity,
    },
    identity,
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig, Event as MdnsEvent},
    noise,
    swarm::{NetworkBehaviour, Swarm},
    tcp::tokio::Transport as TcpTransport,
    yamux::YamuxConfig,
    dns::tokio::Transport as DnsTransport,
    PeerId, Transport,
};
use std::error::Error;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "P2PEvent")]
pub struct P2PNetwork {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
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

    // Create TCP transport
    let tcp = TcpTransport::new(libp2p::tcp::Config::default().nodelay(true));

    // DNS over TCP
    let transport = DnsTransport::system(tcp)?
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&id_keys)?)
        .multiplex(YamuxConfig::default())
        .boxed();

    // Gossipsub setup
    let gossipsub_config = GossipsubConfigBuilder::default()
        .build()
        .expect("Valid gossipsub config");

    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )?;

    gossipsub.subscribe(&IdentTopic::new("yuki"))?;

    // mDNS setup
    let mdns = Mdns::new(MdnsConfig::default(), peer_id)?;

    let behaviour = P2PNetwork { gossipsub, mdns };

    Ok(Swarm::new(transport, behaviour, peer_id))
}
