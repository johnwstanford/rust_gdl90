use crate::StratusGDL90;

#[test]
fn traffic_report_from_udp_packet() -> Result<(), &'static str> {

    const UDP_PACKET: [u8; 32] = [
        0x7E, 0x14, 0x00, 0xA1, 0x09, 0x31, 0x17, 0x9C,
        0xFB, 0xB9, 0xCF, 0x03, 0x46, 0xD9, 0x89, 0x1C,
        0x70, 0x11, 0xCE, 0x03, 0x41, 0x41, 0x4C, 0x32,
        0x30, 0x36, 0x35, 0x20, 0x00, 0xD6, 0xF6, 0x7E
    ];

    let report = StratusGDL90::from_udp_packet(&UDP_PACKET)?;
    let report = report.into_traffic_report().unwrap();

    assert_eq!(report.callsign.as_str(), "AAL2065");
    assert_eq!(report.latitude_deg, 33.20607);
    assert_eq!(report.longitude_deg, -98.706604);
    assert_eq!(report.participant_address, 0xA10931);

    Ok(())
}