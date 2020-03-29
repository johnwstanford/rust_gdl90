
use crate::metar::METAR;

#[test]
fn decode_metars() {
	
	let metar = METAR::from_string("KTKI 130853Z AUTO 00000KT 10SM CLR 16/14 A3008 RMK AO2 SLP183 T01610144 53001");

	println!("{:?}", metar);

}