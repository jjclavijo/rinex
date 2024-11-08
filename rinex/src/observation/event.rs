use crate::{ epoch, fmt_rinex, ground_position::GroundPosition, hardware::Antenna, marker::{GeodeticMarker, MarkerType}, prelude::*, EpochFlag, Header
    //reader::BufferedReader,
    //header::ParsingError
};

use std::{collections::BTreeMap, str::FromStr};
use hifitime::TimeScale;
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

/// Event Record content, sorted by [`Epoch`]
/// Should implement Split and Merge for this.
pub type Record = BTreeMap<
    (Epoch, EpochFlag),
    (
        Option<f64>,
        Event,
    ),
>;

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

/// Formats one epoch according to standard definitions
pub(crate) fn fmt_epoch(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &Event,
    header: &Header,
) -> String {
    if header.version.major < 3 {
        fmt_epoch_v2(epoch, flag, clock_offset, data)
    } else {
        fmt_epoch_v3(epoch, flag, clock_offset, data)
    }
}

fn fmt_epoch_v3(
    epoch: Epoch,
    flag: EpochFlag,
    clock_offset: &Option<f64>,
    data: &Event,
) -> String {
    let mut lines = String::with_capacity(128);

    //TODO: figure a way to compute Event length without formatting
    let evt_text = format!("{}",data);

    let evt_length = match evt_text.trim_end() {
        "" => 0,
        _ => evt_text.trim_end().split("\n").collect::<Vec<_>>().len()
    };

    lines.push_str(&format!(
        "> {}  {} {:2}",
        epoch::format(epoch, crate::types::Type::ObservationData, 3),
        flag,
        evt_length
    ));

    if let Some(data) = clock_offset {
        lines.push_str(&format!("{:13.4}", data));
    }

    lines.push('\n');
    lines.truncate(lines.trim_end().len());
    lines
}

fn fmt_epoch_v2(
    _epoch: Epoch,
    _flag: EpochFlag,
    _clock_offset: &Option<f64>,
    _data: &Event,
) -> String {
    panic!("v2 event parsing NotImplemented")
}



pub mod mixed {

    //
    // The mixing of observations and events is done building a BTreeMap
    // That wraps References to the original data.
    //
    // Event and Observation Records are meant to be managed individually
    //

    use std::collections::{BTreeMap, HashMap};

    use hifitime::Epoch;
    use thiserror::Error;

    use crate::{observation::{self, ObservationData}, prelude::SV, EpochFlag};

    use super::{Event, Observable};
    use crate::observation::event;

    #[derive(Debug)]
    pub enum ObservationOrEvent<'a> {
        Observation( 
            &'a( 
                Option<f64>, 
                BTreeMap<SV, HashMap<Observable, ObservationData>>,
            )
        ),
        Event( &'a( Option<f64>,Event ) )
    }

    pub type Record<'a> = BTreeMap<
        (Epoch, EpochFlag),
        ObservationOrEvent<'a>,
    >;


    #[derive(Error, Debug)]
    pub enum Error {
        #[error("failed observation and event combination")]
        CombinationError
    }


    // The lifetime of the record will be the lifetime of the underlaying
    // objects.
    // 
    // Both observation::Record and event::Record came from with the same
    // lifetime, wrapped in a ObsEvtRecord(...)
    //
    
    pub fn combine_obs_and_evt<'a>(obs: &'a observation::Record, evt: &'a super::Record) ->
        Result<Record<'a>,Error>
    {
        let mut new_btm = BTreeMap::new();

        for (k,map) in obs {
            new_btm.insert(*k,ObservationOrEvent::Observation(map));
        };

        for (k,map) in evt {
            match new_btm.insert(*k,ObservationOrEvent::Event(map))
            {
                None => (),
                Some(_) => return Err(Error::CombinationError)
            };
        };

        Ok(new_btm)
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
