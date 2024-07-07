
use std::error::Error;

use libp2p::{futures::StreamExt, identity, swarm::SwarmEvent, Multiaddr, PeerId, Swarm};
use libp2p::ping::{Ping, PingConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let new_key = identity::Keypair::generate_ed25519();
    let new_peer_id = PeerId::from(new_key.public());
    println!("local peer id is: {:?}", new_peer_id);

    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));
    let transport = libp2p::development_transport(new_key).await.unwrap();
    let mut swarm = Swarm::new(transport, behaviour, new_peer_id);
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
            SwarmEvent::Behaviour(event) => println!("Event received from peer is {:?}", event),
            _ => {},
        }
    }
}