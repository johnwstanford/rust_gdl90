use std::net::UdpSocket;

fn main() -> Result<(), &'static str> {

    let args: Vec<String> = std::env::args().collect();

    let bind_ip = args.get(1).ok_or("Expected bind IP address as first argument")?;

    let mut sock = UdpSocket::bind(bind_ip).unwrap();
    let mut buff = vec![0u8; 1024];

    while let Ok(n) = sock.recv(&mut buff[..]) {
        println!("{:X?}", &buff[..n]);
    }

    Ok(())
}