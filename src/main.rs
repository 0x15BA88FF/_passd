mod routes;

use std::net::{Ipv4Addr, SocketAddr};

fn parse_addr(ip: &str) -> Ipv4Addr {
    ip.parse().unwrap_or_else(|_| {
        eprintln!("Invalid IP address '{}', defaulting to 127.0.0.1", ip);
        Ipv4Addr::new(127, 0, 0, 1)
    })
}

#[tokio::main]
async fn main() {
    let socket_address = SocketAddr::new(parse_addr("127.0.0.1").into(), 7700);

    println!("Server listening on {}", socket_address);
    warp::serve(routes::routes()).run(socket_address).await;
}
