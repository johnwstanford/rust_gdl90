
extern crate regex;
extern crate serde;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Serialize, Deserialize};

lazy_static! {
    static ref METAR_RE: Regex = Regex::new(r"METAR\s(\S{4})\s(\d{2})(\d{2})(\d{2})Z(\sAUTO)?\s(\d{3})(\d{2})G?(\d{2})?KT").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Text {
	METAR{station:String, day:u8, hour:u8, min:u8, is_auto:bool, wind_dir_deg:f32, wind_speed_kts:f32, wind_gust_kts:Option<f32>},
	PIREP,
	TAF,
	Unknown(String),
}

impl Text {

	pub fn from_string(s:String) -> Text {
		if let Some(caps) = METAR_RE.captures(&s) { 
			let station:&str   = caps.get(1).map_or("", |m| m.as_str());
			let res_day        = caps.get(2).map_or("", |m| m.as_str()).parse::<u8>();
			let res_hour       = caps.get(3).map_or("", |m| m.as_str()).parse::<u8>();
			let res_min        = caps.get(4).map_or("", |m| m.as_str()).parse::<u8>();
			let is_auto:bool   = caps.get(5).is_some();
			let res_wind_dir   = caps.get(6).map_or("", |m| m.as_str()).parse::<f32>();
			let res_wind_kts   = caps.get(7).map_or("", |m| m.as_str()).parse::<f32>();
			let res_wind_gust  = caps.get(8).map_or("", |m| m.as_str()).parse::<f32>();
			
			if let (Ok(day), Ok(hour), Ok(min), Ok(wind_dir_deg), Ok(wind_speed_kts)) = (res_day, res_hour, res_min, res_wind_dir, res_wind_kts) {
				let wind_gust_kts = match res_wind_gust {
					Ok(x) => Some(x),
					_     => None,
				};

				Text::METAR{ station: station.to_string(), day, hour, min, is_auto,
					wind_dir_deg, wind_speed_kts, wind_gust_kts }
			} else { Text::Unknown(s) }
		}
		else {
			Text::Unknown(s)
		}
	}

}