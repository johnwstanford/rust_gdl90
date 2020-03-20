
extern crate byteorder;
extern crate serde;

use std::io::{Cursor, Error, ErrorKind};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Serialize, Deserialize};

const LAT_LON_LSB:f32 = 0.000021458;

mod dlac;
pub mod text;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
	pub ground_station_latitude_deg:f32,
	pub ground_station_longitude_deg:f32,
	pub application_data: Vec<Frame>,
}

impl Payload {

	pub fn new(mut payload:Vec<u8>) -> std::io::Result<Payload> {
		let header:Vec<u8> = payload.drain(..8).collect();

		// Decode the header based on Table 2-4, pg. 52 of "Manual for the Universal Access Transceiver"
		let mut rdr = Cursor::new(header);
		let header_msp:u32 = rdr.read_u32::<BigEndian>()?;
		let header_lsp:u32 = rdr.read_u32::<BigEndian>()?;

		// Lat/Lon encoding is the same as described in 3.2.1.5.2.1
		let ground_station_latitude_deg:f32 = {
			let raw:u32 = header_msp >> 9;
			let angular_part:u32 = raw % 2097152;
			match raw >> 21 {
				0 => (angular_part as f32) * LAT_LON_LSB,
				1 => if angular_part == 0 { 90.0 } else { return Err(Error::new(ErrorKind::Other, "Latitude: nonzero angular part in quadrant 1")); },
				2 => return Err(Error::new(ErrorKind::Other, "Latitude in quadrant 2")),
				3 => (angular_part as f32) * LAT_LON_LSB - 90.0,
				_ => return Err(Error::new(ErrorKind::Other, "Invalid value for latitude quadrant")),
			}
		};
		let ground_station_longitude_deg:f32 = {
			let raw:u32 = ((header_msp % 512) * 32768) + (header_lsp >> 17);
			let angular_part:u32 = raw % 4194304;
			match raw >> 23 {
				0 => (angular_part as f32) * LAT_LON_LSB,
				1 => (angular_part as f32) * LAT_LON_LSB - 180.0,
				_ => return Err(Error::new(ErrorKind::Other, "Invalid value for longitude quadrant")),
			}
		};
		// TODO: consider checking to make sure the ground station is within a reasonable radius of the ownship

		let mut application_data:Vec<Frame> = vec![];
		while let Ok(frame) = Frame::new(&mut payload) {  
			application_data.push(frame); 
		}

		Ok(Payload{ ground_station_latitude_deg, ground_station_longitude_deg, application_data })
	}

}

#[derive(Debug, Serialize, Deserialize)]
pub enum Frame {
	NexradPrecipitationImage{ hours:u32, minutes:u32 },
	GenericText{ hours:u32, minutes:u32, text: text::Text },
	Unknown{ id:u32, payload_len:usize, hours:u32, minutes:u32 },
}

impl Frame {

	fn decode_apdu(apdu_header:Vec<u8>, apdu_payload:Vec<u8>) -> std::io::Result<Frame> {
		let header:u32 = {
			let mut rdr = Cursor::new(apdu_header);
			rdr.read_u32::<BigEndian>()?
		};

		// Not sure exactly what these flags mean, but they should all be false
		// for the only two message types we currently know about.  It also seems
		// to be true most almost all unknown messages
		let af:bool = header & 0x80000000 == 0x80000000;
		let gf:bool = header & 0x40000000 == 0x40000000;
		let pf:bool = header & 0x20000000 == 0x20000000;
		let sf:bool = header & 0x00020000 == 0x00020000;
		if af|gf|pf|sf {
			return Err(Error::new(ErrorKind::Other, "Flags in APDU header not as expected"));
		}
		
		let product_id:u32 = (header >> 18) % 2048;
		let hours:u32      = (header >> 10) % 32;
		let minutes:u32    = (header >>  4) % 64;
		if hours   > 23 { return Err(Error::new(ErrorKind::Other, "Invalid hours value"));   }
		if minutes > 59 { return Err(Error::new(ErrorKind::Other, "Invalid minutes value")); }

		match product_id {
			63 => {
				Ok(Frame::NexradPrecipitationImage{ hours, minutes })
			},
			413 => {
				let mut decoder = dlac::Decoder::new();
				for b in apdu_payload { decoder.next(b); }
				Ok(Frame::GenericText{ hours, minutes, text: text::Text::from_string(decoder.get_result()) })
			},
			id => Ok(Frame::Unknown{ id, payload_len: apdu_payload.len(), hours, minutes }),
			// Unkown frame possibilities: winds aloft, SIGMETs, AIRMETs, SUA
		}

		
	}

	pub fn new(payload:&mut Vec<u8>) -> std::io::Result<Frame> {
		// We need at least two bytes to get a frame header, which has the length and frame type
		if payload.len() > 2 {
			let header:u16 = {
				// There might be a more elegant way to do this later, but right now I just want to make sure
				// it works and make sure it takes two bytes off the payload vector
				let bytes:Vec<u8> = payload.drain(..2).collect();
				let mut rdr = Cursor::new(bytes);
				rdr.read_u16::<BigEndian>()?
			};

			// The 9 most significant bits are the length.  The next 3 bytes are reserved and the last 4 are the frame type
			let length:u16     = header >> 7;
			let frame_type:u16 = header % 16;

			// After the frame header, we need at least four more bytes for the APDU header
			// ADPU is "Application Protocol Data Unit"
			if length < 4 { return Err(Error::new(ErrorKind::Other, "No APDU header")); }

			// Frame type 0 is "FIS-B APDU", all other values are reserved or experimental
			if frame_type != 0 { return Err(Error::new(ErrorKind::Other, "Frame type other than FIS-B APDU")); }

			if length == 0 {
				// These zero-length messages are mostly just used for padding
				Err(Error::new(ErrorKind::Other, "Zero-length message"))
			} else if payload.len() < length as usize {
				// The length given in the frame header must be wrong or we got an incomplete message
				Err(Error::new(ErrorKind::Other, "Fewer remaining bytes that the message length"))
			} else {

				// We passed all the tests at the frame level, so go down to the APDU level and see if we get a valid message
				let apdu_header:Vec<u8>  = payload.drain(..4).collect();
				let apdu_payload:Vec<u8> = payload.drain(..((length-4) as usize)).collect();
				
				Frame::decode_apdu(apdu_header, apdu_payload)

			}
		} else {
			Err(Error::new(ErrorKind::Other, "Not enough bytes for a message"))
		}
	}

}

