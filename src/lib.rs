
extern crate byteorder;
extern crate lazy_static;
extern crate serde;

use std::io::{Cursor, Error, ErrorKind};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use serde::{Serialize, Deserialize};

const RAD_PER_DEG:f32 = 3.14159265359 / 180.0;
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
mod test;

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
	Attitude,
}

pub fn interpret_gdl90(id:u8, data:Vec<u8>) -> std::io::Result<StratusGDL90> {

	match id {
		0   => {
			let mut rdr = Cursor::new(data);
		 	Ok(StratusGDL90::Heartbeat{ status_byte1: rdr.read_u8()?, status_byte2: rdr.read_u8()?, 
		 		timestamp:rdr.read_u16::<LittleEndian>()?, msg_count: rdr.read_u16::<BigEndian>()? })
		 },
		2   => Ok(StratusGDL90::Initialization),
		7   => {
			let mut rdr = Cursor::new(data);
			let tor_lsb:u8 = rdr.read_u8()?;
			let tor_2sb:u8 = rdr.read_u8()?;
			let tor_msb:u8 = rdr.read_u8()?;
			let time_of_reception_raw:u32 = (tor_msb as u32 * 65536) + (tor_2sb as u32 * 256) + (tor_lsb as u32);
			let time_of_reception_ns:u32  = time_of_reception_raw * 80;

			let mut buff:Vec<u8> = vec![];
			while let Ok(b) = rdr.read_u8() { buff.push(b); }
			let payload = uplink_data::Payload::new(buff)?;
			Ok(StratusGDL90::UplinkData{ time_of_reception_ns, payload })
		},
		9   => Ok(StratusGDL90::HeightAboveTerrain),
		10  => Ok(StratusGDL90::OwnshipReport(traffic_report::TrafficReport::from_byte_vec(&data)?)),
		11  => {
			let mut rdr = Cursor::new(data);
			let altitude_raw:i16 = rdr.read_i16::<BigEndian>()?;
			Ok(StratusGDL90::OwnshipGeometricAltitude(altitude_raw as f32 * 5.0))
		},
		20  => Ok(StratusGDL90::TrafficReport(traffic_report::TrafficReport::from_byte_vec(&data)?)),
		30  => Ok(StratusGDL90::BasicReport),
		31  => Ok(StratusGDL90::LongReport),
		101 => {
			let mut rdr = Cursor::new(data);
			match rdr.read_u8()? {
				0  => Ok(StratusGDL90::DeviceId),
				1  => Ok(StratusGDL90::Attitude),
				_  => Err(Error::new(ErrorKind::Other, "Unknown sub-ID for message 101, defined in the Foreflight extended spec")),
			}
		},
		_   => Ok(StratusGDL90::Unknown{ id, data }),
	}

}