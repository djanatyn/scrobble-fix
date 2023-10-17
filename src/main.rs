//! My iPod had it's clock reset to 2001, and scrobbles have the incorrect date.
//!
//! Parse the Rockbox scrobbler.log file, identify scrobbles with suspicious dates, and fix them.
//!
//! AUDIOSCROBBLER/1.1 format is documented here:
//! - <https://github.com/Rockbox/rockbox/blob/3c89adbdbdd036baf313786b0694632c8e7e2bb3/apps/plugins/lastfm_scrobbler.c#L29>

use chrono::{DateTime, Days, Local, TimeZone};
use nom::{
    bytes::complete::take_until, character::complete::char, multi::count, sequence::terminated,
    IResult,
};

/// Anything older than this needs an offset applied.
const SCROBBLE_CUTOFF: &str = "2005-01-01T00:00:00Z";

/// Number of days to add to the suspicious scrobbles.
const SCROBBLE_DAYS_OFFSET: u64 = (365 * 22) + 215;

fn main() -> std::io::Result<()> {
    let log = std::fs::read_to_string("scrobbler.log")?;
    let scrobbles: Vec<Scrobble> = log
        .lines()
        .skip(3)
        .map(|i| {
            let (_, scrobble) = parse_scrobble(i).unwrap();
            scrobble
        })
        .collect();
    let cutoff =
        DateTime::parse_from_rfc3339(SCROBBLE_CUTOFF).expect("failed to parse cutoff date");
    for scrobble in scrobbles {
        if scrobble.timestamp < cutoff {
            println!("suspicious: {scrobble:#?}");
            let updated = scrobble
                .timestamp
                .checked_add_days(Days::new(SCROBBLE_DAYS_OFFSET))
                .expect("failed to apply offset");
            println!("updated: {updated:?}");
        }
    }
    Ok(())
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
    track_position: Option<u32>, // TODO: Option<u32>
    song_duration: u32,          // seconds
    rating: Rating,
    timestamp: DateTime<Local>,
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
    let scrobbles: Vec<Scrobble> = log
        .lines()
        .skip(3)
        .map(|i| {
            let (_, scrobble) = parse_scrobble(dbg!(i)).unwrap();
            scrobble
        })
        .collect();
    dbg!(scrobbles);
    Ok(())
}
