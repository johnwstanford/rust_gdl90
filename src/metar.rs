
extern crate regex;
extern crate serde;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Serialize, Deserialize};

// These fields are mandatory, which is why there are no ? qualifiers after the groups
//                              (  1  )  (  2  )(  3  )(  4  )
const STATION_AND_TIME:&str = r"(\S{4})\s(\d{2})(\d{2})(\d{2})Z";

//							   (      5     )
const REPORT_MODIFIER:&str = r"(\sAUTO|\sCOR)?";

// The variable part of the wind group is currently captured, but not decoded.  I haven't found any examples
// to test it on yet where the wind is variable and greater than 6 knots
//                        (                         6                          )
//                           (    7    )(   8   )  (   9   )   (         10    )
const WIND_GROUP:&str = r"(\s(\d{3}|VRB)(\d{2,3})G?(\d{2,3})?KT(\s\d{3}V\d{3})?)?";

//                              (    11    )
const VISIBILITY_GROUP:&str = r"(\s.{1,5}SM)?";

//                                       (               12                       )
//                                           (      13   )      ( 14  )(  15  )
const RUNWAY_VISUAL_RANGE_GROUP:&str = r"(\sR(\d{2}[LRC]?)/[PM]?(\d{4})(V\d{4})?FT)?";

// There are up to three present weather groups and the only way to capture them all is the just repeat this three times
//                                   (                                         16, 20, 24                                                      )
//                                      (17,21,25)(     18, 22, 26        ) (                      19, 23, 27                                 )
const PRESENT_WEATHER_GROUP:&str = r"(\s(\+|-|VC)?(MI|PR|BC|DR|BL|SH|TS|FZ)?(DZ|RA|SN|SG|IC|PL|GR|GS|UP|BR|FG|FU|VA|DU|SA|HZ|PY|PO|SQ|FC|SS|DS))?";

// The specifications don't put a limit on the number of sky condition groups, but the maximum observed in a large set of data is four
//                                 (       28, 30, 32, 34         )
//                                    (     29, 31, 33, 35       )
const SKY_CONDITION_GROUP:&str = r"(\s(\D{3}\d{3}|VV\d{3}|CLR|SKC))?";

//                               (          36         )
//								    (   37  ) (  38   )
const TEMPERATURE_GROUP:&str = r"(\s(M?\d{2})/(M?\d{2}))?";

//							   (    39    )
//                                 (  40 )
const ALTIMETER_GROUP:&str = r"(\sA(\d{4}))?";

lazy_static! {
    static ref METAR_RE: Regex = Regex::new(&format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}", 
    	STATION_AND_TIME, REPORT_MODIFIER, WIND_GROUP, VISIBILITY_GROUP, RUNWAY_VISUAL_RANGE_GROUP,
    	PRESENT_WEATHER_GROUP, PRESENT_WEATHER_GROUP, PRESENT_WEATHER_GROUP,
    	SKY_CONDITION_GROUP, SKY_CONDITION_GROUP, SKY_CONDITION_GROUP, SKY_CONDITION_GROUP,
    	TEMPERATURE_GROUP, ALTIMETER_GROUP)).unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QualityControlFlags {
	Corrected,
	Automated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct METAR {
	// Fields that are always present
	pub station:String, 
	pub day:u8, 
	pub hour:u8, 
	pub min:u8, 

	// Fields that are not always present
	pub quality_control_flags:Vec<QualityControlFlags>, 
	pub wind_dir_deg:Option<f32>, 
	pub wind_spd_kts:Option<f32>, 
	pub wind_gust_kts:Option<f32>,
	pub temperature:Option<f32>,
	pub altimeter:Option<f32>,
	pub dew_point:Option<f32>,
	pub visibility_sm:Option<String>,
	pub sky_condition:Vec<String>,
}

impl METAR {

	pub fn from_string(s:&str) -> Result<METAR, &str> {
		if let Some(caps) = METAR_RE.captures(s) { 
			// Field that are always present; these will return an Err if they aren't found
			let station:&str = caps.get(1).map(|m| m.as_str()).ok_or("No match for station ")?;
			let day:u8       = caps.get(2).map_or("", |m| m.as_str()).parse::<u8>().map_err(|_| "Unable to parse u8")?;
			let hour:u8      = caps.get(3).map_or("", |m| m.as_str()).parse::<u8>().map_err(|_| "Unable to parse u8")?;
			let min:u8       = caps.get(4).map_or("", |m| m.as_str()).parse::<u8>().map_err(|_| "Unable to parse u8")?;

			// Fields that are not always present; these are represented as some kind of optional type in the struct
			let quality_control_flags = match caps.get(5).map_or("", |m| m.as_str()) {
				" AUTO" => vec![QualityControlFlags::Automated],
				" COR"  => vec![QualityControlFlags::Corrected],
				_       => vec![],
			};

			// Group 6 is the entire wind group
			// TODO: implement variable wind group, only applicable if variable winds are greater than 6 knots
			let wind_dir_deg   = match caps.get(7).map(|m| m.as_str()) {
				Some("VRB") => Some(0.0),
				Some(ddd)   => ddd.parse::<f32>().ok(),
				None        => None,
			};
			let wind_spd_kts   = caps.get(8).map_or("", |m| m.as_str()).parse::<f32>().ok();
			let wind_gust_kts  = caps.get(9).map_or("", |m| m.as_str()).parse::<f32>().ok();
			
			// TODO: parse visibility to an f32; will require handling of fractions
			let visibility_sm  = caps.get(15).map(|m| m.as_str().to_owned());

			// TODO: include present weather groups in the struct

			let mut sky_condition:Vec<String> = vec![];
			for idx in &[29, 31, 33, 35] {
				if let Some(skc) = caps.get(*idx).map(|m| m.as_str()) { 
					sky_condition.push(skc.to_string()); 
				}
			}

			let temperature:Option<f32> = caps.get(37).map(|m| m.as_str()).map(|temp_grp| {
				if &temp_grp[..1] == "M" {
					(temp_grp[1..]).parse::<f32>().map(|x| x * -1.0).ok()
				} else {
					temp_grp.parse::<f32>().ok()
				}
			}).unwrap_or(None);

			let dew_point:Option<f32> = caps.get(38).map(|m| m.as_str()).map(|temp_grp| {
				if &temp_grp[..1] == "M" {
					(temp_grp[1..]).parse::<f32>().map(|x| x * -1.0).ok()
				} else {
					temp_grp.parse::<f32>().ok()
				}
			}).unwrap_or(None);

			let altimeter:Option<f32> = caps.get(40).map_or("", |m| m.as_str()).parse::<f32>().ok().map(|a| a / 100.0);

			/*println!("\n{:?}", s);
			for (idx, c) in caps.iter().enumerate() {
				if idx > 30 {
					println!("{} {:?}", idx, c.map(|m| m.as_str()));
				}
			}*/

			Ok(METAR{ station: station.to_string(), day, hour, min, quality_control_flags, wind_dir_deg, wind_spd_kts, wind_gust_kts, 
							visibility_sm, sky_condition, altimeter, temperature, dew_point })
		}
		else {
			Err("Unable to match the text to the METAR regex")
		}
	}

}

