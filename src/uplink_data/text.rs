
extern crate regex;
extern crate serde;

use regex::Regex;
use serde::{Serialize, Deserialize};

use lazy_static::lazy_static;

lazy_static! {
    static ref METAR_RE: Regex = Regex::new(r"METAR\s(.+)").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Text {
	METAR(crate::metar::METAR),
	PIREP,
	TAF,
	Unknown(String),
}

impl Text {

	pub fn from_string(s:String) -> Text {
		if let Some(caps) = METAR_RE.captures(&s) { 
			let metar_str = caps.get(1).map_or("", |m| m.as_str());
			if let Ok(metar) = crate::metar::METAR::from_string(metar_str) {
				Text::METAR(metar)
			} else {
				Text::Unknown(s)
			}
		}
		else {
			Text::Unknown(s)
		}
	}

}