use std::net::UdpSocket;
use rust_gdl90::StratusGDL90;

fn main() -> Result<(), &'static str> {

    let args: Vec<String> = std::env::args().collect();

    let bind_ip = args.get(1).ok_or("Expected bind IP address as first argument")?;
    println!("Bind IP address: {}", bind_ip);

    let sock = UdpSocket::bind(bind_ip).map_err(|_| "Unable to bind")?;
    sock.set_broadcast(true).map_err(|_| "Unable to set broadcast")?;

    let mut buff = vec![0u8; 1024];

    while let Ok((n, _addr)) = sock.recv_from(&mut buff[..]) {

        if let Ok(report) = rust_gdl90::StratusGDL90::from_udp_packet(&buff[..n]) {
            match report {
                StratusGDL90::TrafficReport(traffic) => {
                    println!(
                        "ICAO: 0x{:06X}, lat {:.4} [deg], long {:.4} [deg], alt {} [ft], {}",
                        traffic.participant_address, traffic.latitude_deg, traffic.longitude_deg,
                        traffic.pres_altitude_ft, traffic.callsign,
                    );
                },
                _ => (),
            }
        }

    }

    Ok(())
}