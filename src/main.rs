#![feature(iter_intersperse)]

//! My iPod had it's clock reset to 2001, and scrobbles have the incorrect date.
//!
//! Parse the Rockbox scrobbler.log file, identify scrobbles with suspicious dates, and fix them.
//!
//! AUDIOSCROBBLER/1.1 format is documented here:
//! - <https://github.com/Rockbox/rockbox/blob/3c89adbdbdd036baf313786b0694632c8e7e2bb3/apps/plugins/lastfm_scrobbler.c#L29>

use chrono::{DateTime, Days, FixedOffset, Local, TimeZone};
use nom::{
    bytes::complete::{tag, take_until},
    multi::separated_list1,
    sequence::terminated,
    IResult,
};

/// Anything older than this needs an offset applied.
const SCROBBLE_CUTOFF: &str = "2005-01-01T00:00:00Z";

/// Number of days to add to the suspicious scrobbles.
const SCROBBLE_DAYS_OFFSET: u64 = (365 * 22) + 215;

/// Header for AUDIOSCROBBLER/1.1 format.
const HEADER: &str = r#"#AUDIOSCROBBLER/1.1
#TZ/UNKNOWN
#CLIENT/Rockbox ipodvideo $Revision$
"#;

/// Output scrobbler.log with fixed timestamps.
fn main() -> std::io::Result<()> {
    let cutoff =
        DateTime::parse_from_rfc3339(SCROBBLE_CUTOFF).expect("failed to parse cutoff date");
    let log = std::fs::read_to_string("scrobbler.log")?;
    let scrobbles: String = log
        .lines()
        .skip(3)
        .map(|input| {
            Scrobble::new(input)
                .and_then(|scrobble| scrobble.fix(cutoff).map(|fixed| fixed.to_string()))
        })
        .intersperse(Ok("\n".to_string()))
        .collect::<Result<String, _>>()
        .unwrap();
    Ok(println!("{HEADER}{scrobbles}"))
}

#[derive(Debug)]
enum Rating {
    Listened,
    Skipped,
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Rating::Listened => write!(f, "L"),
            Rating::Skipped => write!(f, "S"),
        }
    }
}

/// Parsed scrobble record.
#[derive(Debug)]
struct Scrobble {
    artist: String,
    album: String,
    track: String,
    track_position: Option<u32>,
    song_duration: u32, // seconds
    rating: Rating,
    timestamp: DateTime<Local>,
    track_id: Option<String>,
}

impl std::fmt::Display for Scrobble {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            [
                &self.artist,
                &self.album,
                &self.track,
                &self
                    .track_position
                    .map_or("".to_string(), |p| p.to_string()),
                &self.song_duration.to_string(),
                &self.rating.to_string(),
                &self.timestamp.timestamp().to_string(),
                &self.track_id.clone().unwrap_or("".to_string())
            ]
            .into_iter()
            .intersperse(&"\t".to_string())
            .cloned()
            .collect::<String>()
        )
    }
}

impl Scrobble {
    /// Parse a scrobble from scrobbler.log
    fn new(input: &str) -> Result<Self, String> {
        let (rest, tokens) = match parse_scrobble_tokens(input) {
            Ok((rest, tokens)) => (rest, tokens),
            Err(e) => Err(e.to_string())?,
        };
        Ok(Scrobble {
            artist: tokens[0].to_string(),
            album: tokens[1].to_string(),
            track: tokens[2].to_string(),
            track_position: match tokens[3] {
                "" => None,
                pos => Some(pos.parse::<u32>().map_err(|e| e.to_string())?),
            },
            song_duration: tokens[4].parse::<u32>().map_err(|e| e.to_string())?,
            rating: match tokens[5] {
                "S" => Rating::Skipped,
                "L" => Rating::Listened,
                _ => Err("failed to parse rating")?,
            },
            timestamp: chrono::Local
                .timestamp_opt(tokens[6].parse::<i64>().map_err(|e| e.to_string())?, 0)
                .unwrap(),
            track_id: match rest {
                "" => None,
                id => Some(id.to_string()),
            },
        })
    }

    /// Adjust the timestamps for suspicious scrobbles.
    fn fix(self, cutoff: DateTime<FixedOffset>) -> Result<Self, String> {
        if self.timestamp > cutoff {
            return Ok(self);
        }
        let updated_timestamp = self
            .timestamp
            .checked_add_days(Days::new(SCROBBLE_DAYS_OFFSET))
            .ok_or("failed to apply offset")?;
        Ok(Self {
            timestamp: updated_timestamp,
            ..self
        })
    }
}

/// Scrobble tokens are separated by tabs. Some fields are empty.
fn parse_scrobble_tokens(input: &str) -> IResult<&str, Vec<&str>> {
    terminated(separated_list1(tag("\t"), take_until("\t")), tag("\t"))(input)
}

#[test]
fn parse_line() -> std::io::Result<()> {
    let log = std::fs::read_to_string("scrobbler.log")?;
    let scrobbles: Result<Vec<Scrobble>, String> = log
        .lines()
        .skip(3)
        .map(|input| Scrobble::new(input))
        .collect();
    Ok(())
}
