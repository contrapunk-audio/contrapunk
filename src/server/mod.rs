pub mod config;
pub mod protocol;
pub mod session;

use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use anyhow::Result;

/// Run the TCP server accept loop.
///
/// Binds to `0.0.0.0:{config.port}`, accepts connections up to
/// `config.max_clients`, and spawns a thread per client running
/// `session::handle_client`.
pub fn run_server(config: &config::ServerConfig) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))?;
    println!("Contrapunk server listening on port {}", config.port);

    let active_clients = Arc::new(AtomicUsize::new(0));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[server] accept error: {}", e);
                continue;
            }
        };

        let peer = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".into());

        let current = active_clients.load(Ordering::SeqCst);
        if current >= config.max_clients {
            eprintln!(
                "[server] rejecting {}: at max clients ({}/{})",
                peer, current, config.max_clients
            );
            drop(stream);
            continue;
        }

        active_clients.fetch_add(1, Ordering::SeqCst);
        let count = active_clients.clone();

        eprintln!(
            "[server] accepted {}: {}/{} clients",
            peer,
            active_clients.load(Ordering::SeqCst),
            config.max_clients
        );

        thread::spawn(move || {
            let _ = session::handle_client(stream);
            let remaining = count.fetch_sub(1, Ordering::SeqCst) - 1;
            eprintln!("[server] client {} done, {} remaining", peer, remaining);
        });
    }

    Ok(())
}
