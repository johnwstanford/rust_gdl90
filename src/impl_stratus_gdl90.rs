use std::io::Cursor;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use crate::{StratusGDL90, traffic_report, uplink_data};
use crate::traffic_report::TrafficReport;

fn parse_error<T>(_:T) -> &'static str {
    "Parse error"
}

impl StratusGDL90 {

    pub fn into_traffic_report(self) -> Option<TrafficReport> {
        match self {
            Self::TrafficReport(r) => Some(r),
            _ => None,
        }
    }

    // A UDP packet received on port 4000
    pub fn from_udp_packet(buff: &[u8]) -> Result<Self, &'static str> {
        if buff.len() < 2 || buff[0] != 0x7e {
            return Err("Failed to parse UDP packet as StratusGDL90");
        }

        let data: &[u8] = &buff[2..];

        match buff[1] {
            0   => {
                let mut rdr = Cursor::new(data);
                Ok(StratusGDL90::Heartbeat{
                    status_byte1: rdr.read_u8().map_err(parse_error)?,
                    status_byte2: rdr.read_u8().map_err(parse_error)?,
                    timestamp:rdr.read_u16::<LittleEndian>().map_err(parse_error)?,
                    msg_count: rdr.read_u16::<BigEndian>().map_err(parse_error)?
                })
            },
            2   => Ok(StratusGDL90::Initialization),
            7   => {
                let mut rdr = Cursor::new(data);
                let tor_lsb:u8 = rdr.read_u8().map_err(parse_error)?;
                let tor_2sb:u8 = rdr.read_u8().map_err(parse_error)?;
                let tor_msb:u8 = rdr.read_u8().map_err(parse_error)?;
                let time_of_reception_raw:u32 = (tor_msb as u32 * 65536) + (tor_2sb as u32 * 256) + (tor_lsb as u32);
                let time_of_reception_ns:u32  = time_of_reception_raw * 80;

                let mut buff:Vec<u8> = vec![];
                while let Ok(b) = rdr.read_u8() { buff.push(b); }
                let payload = uplink_data::Payload::new(buff).map_err(parse_error)?;
                Ok(StratusGDL90::UplinkData{ time_of_reception_ns, payload })
            },
            9   => Ok(StratusGDL90::HeightAboveTerrain),
            10  => Ok(StratusGDL90::OwnshipReport(traffic_report::TrafficReport::from_slice(data).map_err(parse_error)?)),
            11  => {
                let mut rdr = Cursor::new(data);
                let altitude_raw:i16 = rdr.read_i16::<BigEndian>().map_err(parse_error)?;
                Ok(StratusGDL90::OwnshipGeometricAltitude(altitude_raw as f32 * 5.0))
            },
            20  => Ok(StratusGDL90::TrafficReport(traffic_report::TrafficReport::from_slice(&data).map_err(parse_error)?)),
            30  => Ok(StratusGDL90::BasicReport),
            31  => Ok(StratusGDL90::LongReport),
            101 => {
                let mut rdr = Cursor::new(data);
                match rdr.read_u8().map_err(parse_error)? {
                    0  => Ok(StratusGDL90::DeviceId),
                    1  => {
                        let roll_raw:i16  = rdr.read_i16::<BigEndian>().map_err(parse_error)?;
                        let pitch_raw:i16 = rdr.read_i16::<BigEndian>().map_err(parse_error)?;
                        let hdg_raw:u16   = rdr.read_u16::<BigEndian>().map_err(parse_error)?;
                        let ias_raw:u16   = rdr.read_u16::<BigEndian>().map_err(parse_error)?;
                        let tas_raw:u16   = rdr.read_u16::<BigEndian>().map_err(parse_error)?;

                        // Interpret raw values and check for errors
                        let roll_deg  = if roll_raw  < -1800 || roll_raw  > 1800 { None } else { Some((roll_raw  as f32) * 0.1) };
                        let pitch_deg = if pitch_raw < -1800 || pitch_raw > 1800 { None } else { Some((pitch_raw as f32) * 0.1) };

                        let hdg_is_true:bool = hdg_raw & 0x8000 == 0;
                        // TODO: decode heading; will involve bit shifting and u15 to i15 conversion

                        let ias_kts = if ias_raw == 0xFFFF { None } else { Some(ias_raw) };
                        let tas_kts = if tas_raw == 0xFFFF { None } else { Some(tas_raw) };

                        Ok(StratusGDL90::Attitude{ roll_deg, pitch_deg, hdg_is_true, ias_kts, tas_kts })
                    },
                    _  => Err("Unknown sub-ID for message 101, defined in the Foreflight extended spec"),
                }
            },
            _   => Ok(StratusGDL90::Unknown{ id: buff[1], data: data.to_vec() }),
        }
    }

}