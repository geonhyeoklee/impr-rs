
use std::error::Error;

use libp2p::{
    futures::StreamExt, identity, mdns::{Mdns, MdnsConfig, MdnsEvent}, swarm::SwarmEvent, Multiaddr, PeerId, Swarm
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("local peer id is: {:?}", peer_id);

    let transport = libp2p::development_transport(id_keys).await.unwrap();

    // let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));
    let behaviour = Mdns::new(MdnsConfig::default()).await.unwrap();

    let mut swarm = Swarm::new(transport, behaviour, peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

    if let Some(remote_peer) = std::env::args().nth(1) {
        let remote_peer_multiaddr: Multiaddr = remote_peer.parse().unwrap();
        swarm.dial(remote_peer_multiaddr).unwrap();
        println!("Dialed remote peer: {:?}", remote_peer);
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on local address {:?}", address)
            },
            SwarmEvent::Behaviour(MdnsEvent::Discovered(peers)) => {
                for (peer, addr) in peers {
                    println!("discovered {} {}", peer, addr);
                }
            }
            SwarmEvent::Behaviour(MdnsEvent::Expired(expired)) => {
                for (peer, addr) in expired {
                    println!("expired {} {}", peer, addr);
                }
            }
            // SwarmEvent::Behaviour(event) => println!("Event received from peer is {:?}", event),
            _ => {},
        }
    }
}