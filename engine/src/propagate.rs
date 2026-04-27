use sgp4::{Constants, Elements, MinutesSinceEpoch};
use crate::{
    coords,
    error::TrackerError,
    types::{AzEl, EcefPosition, GeodeticPosition, Observer, PassWindow, ScanOptions, TemePosition},
};

fn parse_tle(tle1: &str, tle2: &str) -> Result<(Constants, f64), TrackerError> {
    let elements = Elements::from_tle(None, tle1.trim().as_bytes(), tle2.trim().as_bytes())
        .map_err(|e| TrackerError::TleParse(format!("{e:?}")))?;

    let epoch_s = elements.datetime.and_utc().timestamp() as f64;

    let constants = Constants::from_elements(&elements)
        .map_err(|e| TrackerError::PhysicsInit(format!("{e:?}")))?;

    Ok((constants, epoch_s))
}

/// Propagate already-initialised SGP4 constants to a Unix timestamp and
/// return look angles for the given observer.
/// This is the inner function both `look_angles` and `passes` call in their
/// hot loops, so it takes `Constants` directly to avoid re-parsing the TLE
/// on every timestep.
fn propagate_to(
    constants: &Constants,
    elements_epoch_s: f64,
    unix_ms: f64,
    observer: &Observer,
) -> Result<AzEl, TrackerError> {
    let minutes_since_epoch = (unix_ms / 1_000.0 - elements_epoch_s) / 60.0;

    let prediction = constants
        .propagate(MinutesSinceEpoch(minutes_since_epoch))
        .map_err(|e| TrackerError::Propagation(format!("{e:?}")))?;

    let teme = TemePosition {
        x: prediction.position[0],
        y: prediction.position[1],
        z: prediction.position[2],
    };

    let gmst = coords::get_gmst(unix_ms);
    let ecef = EcefPosition::from_teme(&teme, gmst);

    let geo = GeodeticPosition {
        lat_deg: observer.lat_deg,
        lon_deg: observer.lon_deg,
        alt_m: observer.alt_m,
    };

    Ok(AzEl::from_ecef(&ecef, &geo))
}

/// Return look angles for a satellite at a single instant.
pub fn look_angles(
    tle1: &str,
    tle2: &str,
    unix_ms: f64,
    observer: &Observer,
) -> Result<AzEl, TrackerError> {

    let (constants, epoch_s) = parse_tle(tle1, tle2)?;

    propagate_to(&constants, epoch_s, unix_ms, observer)
}

/// Scan a time window and return every pass where elevation exceeds the threshold.
pub fn passes(
    tle1: &str,
    tle2: &str,
    observer: &Observer,
    options: &ScanOptions,
) -> Result<Vec<PassWindow>, TrackerError> {
    // Validate scan options before doing any real work
    if options.duration_hours <= 0.0 || options.duration_hours > 336.0 {
        return Err(TrackerError::InvalidInput(format!(
            "duration_hours {:.1} must be in (0, 336]", options.duration_hours
        )));
    }

    let (constants, epoch_s) = parse_tle(tle1, tle2)?;

    const STEP_MS: f64 = 15_000.0;
    let end_ms = options.start_ms + options.duration_hours * 3_600_000.0;

    let mut passes: Vec<PassWindow> = Vec::new();
    let mut in_pass = false;
    let mut pass_start_ms = 0.0_f64;
    let mut max_el = f64::NEG_INFINITY;
    let mut max_el_time_ms = 0.0_f64;

    let mut t = options.start_ms;
    while t <= end_ms {
        match propagate_to(&constants, epoch_s, t, observer) {
            Ok(azel) => {
                if azel.elevation >= options.min_elevation_deg {
                    if !in_pass {
                        in_pass = true;
                        pass_start_ms = t;
                        max_el = f64::NEG_INFINITY;
                    }
                    if azel.elevation > max_el {
                        max_el = azel.elevation;
                        max_el_time_ms = t;
                    }
                } else if in_pass {
                    in_pass = false;
                    passes.push(PassWindow {
                        start_ms: pass_start_ms,
                        end_ms: t,
                        max_elevation_deg: max_el,
                        max_el_time_ms,
                    });
                }
            }
            // A propagation failure mid-scan (e.g. orbit decay) closes the
            // current pass rather than aborting the whole search.
            Err(_) => {
                if in_pass {
                    in_pass = false;
                    passes.push(PassWindow {
                        start_ms: pass_start_ms,
                        end_ms: t,
                        max_elevation_deg: max_el,
                        max_el_time_ms,
                    });
                }
            }
        }
        t += STEP_MS;
    }

    // Close a pass still open at the scan window boundary
    if in_pass {
        passes.push(PassWindow {
            start_ms: pass_start_ms,
            end_ms,
            max_elevation_deg: max_el,
            max_el_time_ms,
        });
    }

    Ok(passes)
}