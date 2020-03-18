
extern crate byteorder;
extern crate serde;

use std::io::Cursor;
use std::time::SystemTime;

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Serialize, Deserialize};

const LAT_LON_SCALE:f32 = 2.1457672119140625e-05;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficReport {
	pub status_byte: u8,
	pub participant_address: u32,
	pub latitude_deg: f32,							// TODO: turn into an Option<f32> that can be None based on the NIC flag
	pub longitude_deg: f32,							// TODO: turn into an Option<f32> that can be None based on the NIC flag
	pub pres_altitude_ft: f32,						// TODO: turn into an Option<f32> that is None in the case of 0xFFF
	pub nav_integrity_category: u8,
	pub nav_accuracy_category_for_position: u8,
	pub horz_velocity_kts: f32,						// TODO: turn into an Option<f32> that is None in the case of 0xFFF
	pub vert_velocity_fpm: f32,						// TODO: turn into an Option<f32> that is None in the case of 0x800
	pub track_heading_deg: f32,
	pub emitter_category: EmitterCategory,
	pub callsign:String,
	pub recv_time:SystemTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum EmitterCategory {
	NotAvailable,
	Light,
	Small,
	Large,
	HighVortexLarge,
	Heavy,
	HighlyManeuverable,
	Rotorcraft,
	GliderOrSailplane,
	LighterThanAir,
	Parachutist,
	Ultralight,
	UnmannedAerialVehicle,
	SpaceOrTransatmospheric,
	SurfaceEmergencyVehicle,
	SurfaceServiceVehicle,
	PointObstacle,
	ClusterObstacle,
	LineObstacle,
	ReservedOrUnassigned,
}

impl TrafficReport {

	pub fn new() -> TrafficReport {
		TrafficReport {
			status_byte: 0, participant_address: 0,
			latitude_deg: 0.0, longitude_deg: 0.0,
			pres_altitude_ft: 0.0, nav_integrity_category: 0,
			nav_accuracy_category_for_position: 0, horz_velocity_kts: 0.0,
			vert_velocity_fpm: 0.0, track_heading_deg: 0.0,
			emitter_category: EmitterCategory::NotAvailable, 
			callsign: String::new(),
			recv_time: SystemTime::now(),
		}
	}

	pub fn distance_nm_to(&self, other:&TrafficReport) -> f32 {
		let dist_h:f32 = crate::util::lat_lon_dist_nm(self.latitude_deg, self.longitude_deg, other.latitude_deg, other.longitude_deg);
		let dist_v:f32 = (other.pres_altitude_ft - self.pres_altitude_ft) / crate::FEET_PER_NM;

		(dist_v.powi(2) + dist_h.powi(2)).sqrt()
	}

	pub fn project(&self, dt_sec:f32) -> TrafficReport {

		let dist_m   = self.horz_velocity_kts * (dt_sec / 3600.0) * crate::METERS_PER_NM;
		let brng_rad = self.track_heading_deg * crate::RAD_PER_DEG;

		let delta:f32 = dist_m / crate::R;
		let phi1:f32  = self.latitude_deg * crate::RAD_PER_DEG;
		let lam1:f32  = self.longitude_deg * crate::RAD_PER_DEG;

		let phi2:f32 = (phi1.sin()*delta.cos() + phi1.cos()*delta.sin()*brng_rad.cos()).asin();
		let lam2:f32 = lam1 + (brng_rad.sin()*delta.sin()*phi1.sin()).atan2(delta.cos() - phi1.sin()*phi2.sin());

		let latitude_deg:f32     = phi2 / crate::RAD_PER_DEG;
		let longitude_deg:f32    = lam2 / crate::RAD_PER_DEG;
		let pres_altitude_ft:f32 = self.pres_altitude_ft + self.vert_velocity_fpm*(dt_sec/60.0);

		TrafficReport {
			status_byte:         self.status_byte, 
			participant_address: self.participant_address,
			latitude_deg, longitude_deg, pres_altitude_ft, 
			nav_integrity_category:             self.nav_integrity_category,
			nav_accuracy_category_for_position: self.nav_accuracy_category_for_position, 
			horz_velocity_kts: self.horz_velocity_kts, 
			vert_velocity_fpm: self.vert_velocity_fpm, 
			track_heading_deg: self.track_heading_deg,
			emitter_category:  self.emitter_category, 
			callsign:          self.callsign.clone(),
			recv_time:         self.recv_time,
		}
	}

	pub fn from_byte_vec(data:&Vec<u8>) -> std::io::Result<TrafficReport> {
		let mut rdr = Cursor::new(data);
		let status_byte:u8              = rdr.read_u8()?;
		
		let participant_address_msb:u16 = rdr.read_u16::<BigEndian>()?;
		let participant_address_lsb:u8  = rdr.read_u8()?;
		let participant_address:u32     = (participant_address_msb as u32 * 256) + (participant_address_lsb as u32);

		let latitude_raw_msb:i16 = rdr.read_i16::<BigEndian>()?;
		let latitude_raw_lsb:u8  = rdr.read_u8()?;
		let latitude_raw:i32     = (latitude_raw_msb as i32 * 256) + (latitude_raw_lsb as i32);
		let latitude_deg:f32     = (latitude_raw as f32) * LAT_LON_SCALE;

		let longitude_raw_msb:i16 = rdr.read_i16::<BigEndian>()?;
		let longitude_raw_lsb:u8  = rdr.read_u8()?;
		let longitude_raw:i32     = (longitude_raw_msb as i32 * 256) + (longitude_raw_lsb as i32);
		let longitude_deg:f32     = (longitude_raw as f32) * LAT_LON_SCALE;

		let pres_altitude_misc_raw:u16 = rdr.read_u16::<BigEndian>()?;
		let pres_altitude_raw:u16      = pres_altitude_misc_raw >> 4;
		let pres_altitude_ft:f32       = ((pres_altitude_raw as f32) * 25.0) - 1000.0;
		// TODO: decode miscellaneous indicators

		let nic_nacp_raw:u8                       = rdr.read_u8()?;
		let nav_integrity_category:u8             = nic_nacp_raw >> 4;
		let nav_accuracy_category_for_position:u8 = nic_nacp_raw & 0x0F;

		let velocity_raw_msb:u16   =  rdr.read_u16::<BigEndian>()?;
		let velocity_raw_lsb:u8    =  rdr.read_u8()?;    
		let velocity_raw:u32       = (velocity_raw_msb as u32 * 256) + (velocity_raw_lsb as u32);
		let horz_velocity_raw:u32  =  velocity_raw >> 12;
		let vert_velocity_raw:u32  =  velocity_raw &  0x000007FF;
		let vert_velocity_pos:bool = (velocity_raw &  0x00000800) == 0;
		let horz_velocity_kts:f32  =  horz_velocity_raw as f32;
		let vert_velocity_fpm:f32  = if vert_velocity_pos {  (vert_velocity_raw               as f32) *  64.0 }
									 else                 { ((vert_velocity_raw ^ 0x000007FF) as f32) * -64.0 }; 

		let track_heading_raw:u8   = rdr.read_u8()?;
		let track_heading_deg:f32  = (track_heading_raw as f32) * (360.0 / 256.0);    

		let emitter_category:EmitterCategory = match rdr.read_u8()? {
			0  => EmitterCategory::NotAvailable,
			1  => EmitterCategory::Light,
			2  => EmitterCategory::Small,
			3  => EmitterCategory::Large,
			4  => EmitterCategory::HighVortexLarge,
			5  => EmitterCategory::Heavy,
			6  => EmitterCategory::HighlyManeuverable,
			7  => EmitterCategory::Rotorcraft,
			9  => EmitterCategory::GliderOrSailplane,
			10 => EmitterCategory::LighterThanAir,
			11 => EmitterCategory::Parachutist,
			12 => EmitterCategory::Ultralight,
			14 => EmitterCategory::UnmannedAerialVehicle,
			15 => EmitterCategory::SpaceOrTransatmospheric,
			17 => EmitterCategory::SurfaceEmergencyVehicle,
			18 => EmitterCategory::SurfaceServiceVehicle,
			19 => EmitterCategory::PointObstacle,
			20 => EmitterCategory::ClusterObstacle,
			21 => EmitterCategory::LineObstacle,
			_  => EmitterCategory::ReservedOrUnassigned,
		};

		let mut callsign:String = String::new();
		for _ in 0..8 {
			let c = rdr.read_u8()?;
			if c != 0x00 && c != 0x20 {
				callsign.push(c as char);
			}
		}

		let recv_time = SystemTime::now();

		Ok(TrafficReport{ status_byte, participant_address, latitude_deg, longitude_deg, pres_altitude_ft,
			nav_integrity_category, nav_accuracy_category_for_position, horz_velocity_kts, vert_velocity_fpm,
			track_heading_deg, emitter_category, callsign, recv_time})
	}

}
