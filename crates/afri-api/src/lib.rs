#[derive(Clone, Debug)]
pub struct ApiServer {
    pub host: String,
    pub port: u16,
}

impl ApiServer {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn run(&self) {
        println!("API running on {}:{}", self.host, self.port);
    }
}
