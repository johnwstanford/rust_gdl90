use std::net::UdpSocket;

fn main() -> Result<(), &'static str> {

    let args: Vec<String> = std::env::args().collect();

    let bind_ip = args.get(1).ok_or("Expected bind IP address as first argument")?;
    println!("Bind IP address: {}", bind_ip);

    let sock = UdpSocket::bind(bind_ip).map_err(|_| "Unable to bind")?;
    sock.set_broadcast(true).map_err(|_| "Unable to set broadcast")?;

    let mut buff = vec![0u8; 1024];

    while let Ok((n, addr)) = sock.recv_from(&mut buff[..]) {
        println!("{:?}: {:X?}", addr, &buff[..n]);
    }

    Ok(())
}