use crate::{
    fmt_rinex, ground_position::GroundPosition, hardware::Antenna, marker::{GeodeticMarker, MarkerType}, EpochFlag
    //reader::BufferedReader,
    //header::ParsingError
};

use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::Serialize;


#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("Number of lines in the event ({0}) doesn't match the declared one ({1})")]
    EventLengthError(u16,u16),
    #[error("failed to parse \"{0}\" coordinates from \"{1}\"")]
    CoordinatesParsing(String, String),
    #[error("parsing of {0} events is not implemented")]
    NotImplemented(EpochFlag),
}

/// Some event flags can be followed by header information
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Event {
    /// comments extracted from `header` section
    pub comments: Vec<String>,
    /// optionnal [GeodeticMarker]
    pub geodetic_marker: Option<GeodeticMarker>,
    /// Station approximate coordinates
    pub ground_position: Option<GroundPosition>,
    /// Optionnal Receiver Antenna information
    #[cfg_attr(feature = "serde", serde(default))]
    pub rcvr_antenna: Option<Antenna>,
}

impl Event {
    pub fn new(reader: &str) -> Result<Event, ParsingError> {
        let mut comments: Vec<String> = Vec::new();
        let mut geodetic_marker = Option::<GeodeticMarker>::None;
        let mut rcvr_antenna: Option<Antenna> = None;
        let mut ground_position: Option<GroundPosition> = None;

        // iterate on a line basis
        let lines = reader.lines();
        for l in lines {
            let line = l;//.unwrap();
            if line.len() < 60 {
                continue; // --> invalid header content
            }
            let (content, marker) = line.split_at(60);
            ///////////////////////////////
            // [0] END OF HEADER
            //     --> done parsing
            ///////////////////////////////
            if marker.trim().eq("END OF HEADER") {
                break;
            }
            ///////////////////////////////
            // [0*] COMMENTS
            ///////////////////////////////
            if marker.trim().eq("COMMENT") {
                // --> storing might be useful
                comments.push(content.trim().to_string());
                continue;
            ///////////////////////////////////////
            // ==> from now on
            // RINEX standard / shared attributes
            ///////////////////////////////////////
            } else if marker.contains("MARKER NAME") {
                let name = content.split_at(20).0.trim();
                geodetic_marker = Some(GeodeticMarker::default().with_name(name));
            } else if marker.contains("MARKER NUMBER") {
                let number = content.split_at(20).0.trim();
                if let Some(ref mut marker) = geodetic_marker {
                    *marker = marker.with_number(number);
                }
            } else if marker.contains("MARKER TYPE") {
                let code = content.split_at(20).0.trim();
                if let Ok(mtype) = MarkerType::from_str(code) {
                    if let Some(ref mut marker) = geodetic_marker {
                        marker.marker_type = Some(mtype);
                    }
                }
            } else if marker.contains("APPROX POSITION XYZ") {
                // station base coordinates
                let items: Vec<&str> = content.split_ascii_whitespace().collect();
                let x = items[0].trim();
                let x = f64::from_str(x).or(Err(ParsingError::CoordinatesParsing(
                    String::from("APPROX POSITION X"),
                    x.to_string(),
                )))?;

                let y = items[1].trim();
                let y = f64::from_str(y).or(Err(ParsingError::CoordinatesParsing(
                    String::from("APPROX POSITION Y"),
                    y.to_string(),
                )))?;

                let z = items[2].trim();
                let z = f64::from_str(z).or(Err(ParsingError::CoordinatesParsing(
                    String::from("APPROX POSITION Z"),
                    z.to_string(),
                )))?;

                ground_position = Some(GroundPosition::from_ecef_wgs84((x, y, z)));
            } else if marker.contains("ANT # / TYPE") {
                let (model, rem) = content.split_at(20);
                let (sn, _) = rem.split_at(20);
                if let Some(a) = &mut rcvr_antenna {
                    *a = a.with_model(model.trim()).with_serial_number(sn.trim());
                } else {
                    rcvr_antenna = Some(
                        Antenna::default()
                            .with_model(model.trim())
                            .with_serial_number(sn.trim()),
                    );
                }
            } else if marker.contains("ANTENNA: DELTA X/Y/Z") {
                // Antenna Base/Reference Coordinates
                let items: Vec<&str> = content.split_ascii_whitespace().collect();

                let x = items[0].trim();
                let x = f64::from_str(x).or(Err(ParsingError::CoordinatesParsing(
                    String::from("ANTENNA DELTA X"),
                    x.to_string(),
                )))?;

                let y = items[1].trim();
                let y = f64::from_str(y).or(Err(ParsingError::CoordinatesParsing(
                    String::from("ANTENNA DELTA Y"),
                    y.to_string(),
                )))?;

                let z = items[2].trim();
                let z = f64::from_str(z).or(Err(ParsingError::CoordinatesParsing(
                    String::from("ANTENNA DELTA Z"),
                    z.to_string(),
                )))?;

                if let Some(ant) = &mut rcvr_antenna {
                    *ant = ant.with_base_coordinates((x, y, z));
                } else {
                    rcvr_antenna = Some(Antenna::default().with_base_coordinates((x, y, z)));
                }
            } else if marker.contains("ANTENNA: DELTA H/E/N") {
                // Antenna H/E/N eccentricity components
                let (h, rem) = content.split_at(15);
                let (e, rem) = rem.split_at(15);
                let (n, _) = rem.split_at(15);
                if let Ok(h) = f64::from_str(h.trim()) {
                    if let Ok(e) = f64::from_str(e.trim()) {
                        if let Ok(n) = f64::from_str(n.trim()) {
                            if let Some(a) = &mut rcvr_antenna {
                                *a = a
                                    .with_height(h)
                                    .with_eastern_component(e)
                                    .with_northern_component(n);
                            } else {
                                rcvr_antenna = Some(
                                    Antenna::default()
                                        .with_height(h)
                                        .with_eastern_component(e)
                                        .with_northern_component(n),
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(Event {
            comments,
            geodetic_marker,
            ground_position,
            rcvr_antenna,
        })
    }
}


impl std::fmt::Display for Event {
    /// `Event` formatter, mainly for RINEX file production purposes
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        if let Some(marker) = &self.geodetic_marker {
            writeln!(f, "{}", fmt_rinex(&marker.name, "MARKER NAME"))?;
            if let Some(number) = marker.number() {
                writeln!(f, "{}", fmt_rinex(&number, "MARKER NUMBER"))?;
            }
        }

        // APRIORI POS
        if let Some(position) = self.ground_position {
            writeln!(
                f,
                "{}",
                fmt_rinex(&format!("{:X}", position), "APPROX POSITION XYZ")
            )?;
        }

        // ANT
        if let Some(antenna) = &self.rcvr_antenna {
            // writeln!(
            //     f,
            //     "{}",
            //     fmt_rinex(
            //         &format!("{:<20}{}", antenna.model, antenna.sn),
            //         "ANT # / TYPE"
            //     )
            // )?;
            if let Some(coords) = &antenna.coords {
                writeln!(
                    f,
                    "{}",
                    fmt_rinex(
                        &format!("{:14.4}{:14.4}{:14.4}", coords.0, coords.1, coords.2),
                        "APPROX POSITION XYZ"
                    )
                )?;
            }
            writeln!(
                f,
                "{}",
                fmt_rinex(
                    &format!(
                        "{:14.4}{:14.4}{:14.4}",
                        antenna.height.unwrap_or(0.0),
                        antenna.eastern.unwrap_or(0.0),
                        antenna.northern.unwrap_or(0.0)
                    ),
                    "ANTENNA: DELTA H/E/N"
                )
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Event;
    #[test]
    fn parse_event() {
        // test parsing event info results in same output
        let lineas = 
"STOP_04                                                     MARKER NAME
  2715785.3091 -4504346.1482 -3595759.8391                  APPROX POSITION XYZ
        1.4885        0.0000        0.0000                  ANTENNA: DELTA H/E/N
";

        let evt = Event::new(lineas).expect("parsing error");
        assert_eq!(format!("{}",evt),lineas)
    }
}
