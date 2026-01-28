/// Configuration for the Contrapunk TCP server.
pub struct ServerConfig {
    /// TCP port to listen on.
    pub port: u16,
    /// Maximum number of simultaneous client connections.
    pub max_clients: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 9900,
            max_clients: 10,
        }
    }
}
