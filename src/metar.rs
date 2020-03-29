
extern crate regex;
extern crate serde;

use std::io::{Error, ErrorKind, Result};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Serialize, Deserialize};

lazy_static! {
    //                                         Station  Day    Hour   Min   Z            Dir.   Speed      Gust                                   
    static ref METAR_RE: Regex = Regex::new(r"(\S{4})\s(\d{2})(\d{2})(\d{2})Z(\sAUTO)?\s(\d{3})(\d{2,3})G?(\d{2,3})?KT").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct METAR {
	pub station:String, 
	pub day:u8, 
	pub hour:u8, 
	pub min:u8, 
	pub is_auto:bool, 
	pub wind_dir_deg:f32, 
	pub wind_speed_kts:f32, 
	pub wind_gust_kts:Option<f32>
}

impl METAR {

	pub fn from_string(s:&str) -> Result<METAR> {
		if let Some(caps) = METAR_RE.captures(s) { 
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

				let ans = METAR{ station: station.to_string(), day, hour, min, is_auto, wind_dir_deg, wind_speed_kts, wind_gust_kts };
				Ok(ans)
			} else { 
				Err(Error::new(ErrorKind::Other, "Regex matches the text, but unable to convert group captures into a METAR"))
			}
		}
		else {
			Err(Error::new(ErrorKind::Other, "Unable to match the text to the METAR regex"))
		}
	}

}

