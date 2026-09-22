#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub address: String,
}

impl Peer {
    pub fn new(id: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            address: address.into(),
        }
    }
}

pub fn discover_peers() -> Vec<Peer> {
    vec![
        Peer::new("node-01", "127.0.0.1:7001"),
        Peer::new("node-02", "127.0.0.1:7002"),
    ]
}
