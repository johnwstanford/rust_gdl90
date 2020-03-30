
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

use crate::metar::METAR;

#[derive(Debug, Deserialize)]
struct JsonMetar {
	raw_text: String,
	station_id: String,
	metar_type: String,
	observation_time: String,
	altim_in_hg: Option<f32>,
	wind_dir_degrees: Option<f32>,
	wind_speed_kt: Option<f32>,
	wind_gust_kt: Option<f32>,
	temp_c: Option<f32>,
	dewpoint_c: Option<f32>,
}

#[test]
fn decode_metars() {
	
	let file = File::open("./data/metar_data.json").unwrap();
	let reader = BufReader::new(file);

	let metar_data:Vec<JsonMetar> = serde_json::from_reader(reader).unwrap();

	for json_metar in metar_data.iter().take(10) {
	
		let metar = METAR::from_string(&json_metar.raw_text).unwrap();

		// Compare station
		assert_eq!(metar.station,       json_metar.station_id);
		assert_eq!(metar.wind_dir_deg,  json_metar.wind_dir_degrees);
		assert_eq!(metar.wind_spd_kts,  json_metar.wind_speed_kt);
		assert_eq!(metar.wind_gust_kts, json_metar.wind_gust_kt);

		match (metar.temperature, json_metar.temp_c) {
			(Some(_), None   ) => {
				println!("{:?}", json_metar.raw_text);
				println!("{:?}", json_metar);
				println!("{:?}", metar);
				panic!("One METAR has temperature and the other doesn't")
			},
			(None,    Some(_)) => {
				println!("{:?}", json_metar.raw_text);
				println!("{:?}", json_metar);
				println!("{:?}", metar);
				panic!("One METAR has temperature and the other doesn't")
			},
			(Some(x), Some(y)) => assert!((x-y).abs() < 0.6),
			(None,    None   ) => {},
		}

		match (metar.altimeter, json_metar.altim_in_hg) {
			(Some(_), None   ) => panic!("One METAR has altimeter setting and the other doesn't"),
			(None,    Some(_)) => panic!("One METAR has altimeter setting and the other doesn't"),
			(Some(x), Some(y)) => assert!((x-y).abs() < 0.6),
			(None,    None   ) => {},
		}
	}


}