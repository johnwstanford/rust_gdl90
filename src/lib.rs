
extern crate byteorder;
extern crate lazy_static;
extern crate serde;

use serde::{Serialize, Deserialize};

const RAD_PER_DEG:f32 = std::f32::consts::PI / 180.0;
const METERS_PER_NM:f32 = 1852.0;
const FEET_PER_NM:f32   = 6076.12;

const R:f32 = 6.371e6;

// Preprocessing steps that need to be applied to raw UDP packets before being consumed by the
// rest of the library
pub mod preprocessing;

// Modules related to messages that are complicated enough to require their own modules
pub mod traffic_report;
pub mod uplink_data;

pub mod metar;

// Useful utilities that aren't really GDL90-specific, but are needed in more than one place
pub mod util;

#[cfg(test)]
pub mod tests;

mod impl_stratus_gdl90;

// This is called StratusGDL90 because it includes extensions to the standard GDL90 specification
// transmitted by Stratus units and used by ForeFlight
#[derive(Debug, Serialize, Deserialize)]
pub enum StratusGDL90 {
	Heartbeat{status_byte1:u8, status_byte2:u8, timestamp:u16, msg_count:u16},
	Initialization,
	UplinkData{ time_of_reception_ns:u32, payload:uplink_data::Payload },
	HeightAboveTerrain,
	OwnshipReport(traffic_report::TrafficReport),
	OwnshipGeometricAltitude(f32),
	TrafficReport(traffic_report::TrafficReport),
	BasicReport,
	LongReport,
	Unknown{ id:u8, data:Vec<u8> },
	DeviceId,
	Attitude{ roll_deg:Option<f32>, pitch_deg:Option<f32>, hdg_is_true:bool, ias_kts:Option<u16>, tas_kts:Option<u16> },
}

