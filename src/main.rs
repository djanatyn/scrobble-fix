#![feature(iter_intersperse)]

//! My iPod had it's clock reset to 2001, and scrobbles have the incorrect date.
//!
//! Parse the Rockbox scrobbler.log file, identify scrobbles with suspicious dates, and fix them.
//!
//! AUDIOSCROBBLER/1.1 format is documented here:
//! - <https://github.com/Rockbox/rockbox/blob/3c89adbdbdd036baf313786b0694632c8e7e2bb3/apps/plugins/lastfm_scrobbler.c#L29>

use std::fmt::Display;

use chrono::{DateTime, Days, FixedOffset, Local, TimeZone};
use nom::{
    bytes::complete::take_until, character::complete::char, multi::count, sequence::terminated,
    IResult,
};

/// Anything older than this needs an offset applied.
const SCROBBLE_CUTOFF: &str = "2005-01-01T00:00:00Z";

/// Number of days to add to the suspicious scrobbles.
const SCROBBLE_DAYS_OFFSET: u64 = (365 * 22) + 215;

fn fix_scrobble(cutoff: DateTime<FixedOffset>, scrobble: Scrobble) -> Scrobble {
    if scrobble.timestamp > cutoff {
        return scrobble;
    }
    let updated_timestamp = scrobble
        .timestamp
        .checked_add_days(Days::new(SCROBBLE_DAYS_OFFSET))
        .expect("failed to apply offset");
    return Scrobble {
        timestamp: updated_timestamp,
        ..scrobble
    };
}

fn main() -> std::io::Result<()> {
    let cutoff =
        DateTime::parse_from_rfc3339(SCROBBLE_CUTOFF).expect("failed to parse cutoff date");
    let log = std::fs::read_to_string("scrobbler.log")?;
    let scrobbles: String = log
        .lines()
        .skip(3)
        .map(|input| {
            Scrobble::new(input).and_then(|scrobble| Ok(fix_scrobble(cutoff, scrobble).to_string()))
        })
        .intersperse(Ok("\n".to_string()))
        .collect::<Result<String, _>>()
        .unwrap();
    Ok(println!("{scrobbles}"))
}

#[derive(Debug)]
enum Rating {
    Listened,
    Skipped,
}

#[derive(Debug)]
struct Scrobble {
    artist: String,
    album: String,
    track: String,
    track_position: Option<u32>,
    song_duration: u32, // seconds
    rating: Rating,
    timestamp: DateTime<Local>,
}

impl std::fmt::Display for Scrobble {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}") // TODO
    }
}

impl Scrobble {
    fn new(input: &str) -> Result<Self, String> {
        match parse_scrobble(input) {
            Ok((_, scrobble)) => Ok(scrobble),
            Err(e) => Err(e.to_string()),
        }
    }
}

fn parse_scrobble_token(input: &str) -> IResult<&str, &str> {
    terminated(take_until("\t"), char('\t'))(input)
}

fn parse_scrobble(input: &str) -> IResult<&str, Scrobble> {
    let (rest, tokens) = count(parse_scrobble_token, 7)(input)?;
    Ok((
        rest,
        Scrobble {
            artist: tokens[0].to_string(),
            album: tokens[1].to_string(),
            track: tokens[2].to_string(),
            track_position: match tokens[3] {
                "" => None,
                pos => Some(pos.parse::<u32>().expect("failed to parse track position")),
            },
            song_duration: tokens[4]
                .parse::<u32>()
                .expect("failed to parse song duration"),
            rating: match tokens[5] {
                "S" => Rating::Skipped,
                "L" => Rating::Listened,
                _ => panic!("failed to parse rating"),
            },
            timestamp: chrono::Local
                .timestamp_opt(
                    tokens[6].parse::<i64>().expect("failed to parse timestamp"),
                    0,
                )
                .unwrap(),
        },
    ))
}

#[test]
fn parse_line() -> std::io::Result<()> {
    let log = std::fs::read_to_string("scrobbler.log")?;
    let scrobbles: Vec<Scrobble> = log.lines().skip(3).map(Scrobble::new).collect();
    dbg!(scrobbles);
    Ok(())
}
