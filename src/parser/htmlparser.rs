//! A monolithic block of code that handles parsing of an HTML SIS listing table.

use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use crate::models::*;
use scraper::{Html, Selector};
use std::str::FromStr;
use regex::Regex;
use lazy_static::lazy_static;
use crate::parser::{CourseDBParseError, CourseDBParser};

struct Selectors {
    pub tr: Selector,
    pub td: Selector,
}

lazy_static! {
    static ref SELECTORS: Selectors = {
        Selectors {
            tr: Selector::parse("tr").unwrap(),
            td: Selector::parse("td").unwrap(),
        }
    };

    static ref DAY_REGEX: Regex = {
        Regex::new(r"^[MTWRF]*$").unwrap()
    };
}

pub struct HtmlParser;

impl CourseDBParser for HtmlParser {
    fn parse(&self, input: &str) -> Result<CourseDB, CourseDBParseError> {
        Ok(parse_html_lossy(input))
    }
}

fn parse_html_lossy(input: &str) -> CourseDB {
    let mut db = CourseDB {
        courses: Vec::new(),
    };

    let fragment = input.replace("\n", "");

    let html = Html::parse_fragment(fragment.as_str());



    let mut last_course = 0usize;

    for tr in html.select(&SELECTORS.tr) {
        let tds = tr.select(&SELECTORS.td).collect::<Vec<_>>();
        if tds.len() == 0 {
            continue;
        }
        if tds.len() < 22 {
            eprintln!("Unexpected HTML at {}", tr.html());
        }
        let section_str = tds[4].text().next().unwrap_or("00");
        if section_str == "H01" {
            continue;
        }
        let course = if section_str == "01" {
            // new course
            db.courses.push(Course {
                name: tds[7].text().next().unwrap().to_string(),
                dept: tds[2].text().next().unwrap().to_string(),
                num: u16::from_str(tds[3].text().next().unwrap()).unwrap(),
                sections: vec![],
            });
            last_course = db.courses.len() - 1;
            db.courses.get_mut(last_course).unwrap()
        } else {
            db.courses.get_mut(last_course).unwrap()
        };
        let section_len = course.sections.len();
        let is_period = !section_str.as_bytes()[0].is_ascii_digit();
        let section = if is_period {
            course.sections.get_mut(section_len - 1).unwrap()
        } else {
            course.sections.push(Section {
                crn: u32::from_str(tds[1].text().next().unwrap()).expect("Reading u32 for crn"),
                num: u8::from_str(section_str).expect("Reading section"),
                notes: vec![],
                periods: vec![],
            });
            course.sections.get_mut(section_len).unwrap()
        };

        let day_str = match tds[8]
            .text()
            .filter(|k| k != &"TBA" && DAY_REGEX.is_match(k))
            .next() {
            Some(s) => s,
            None => {
                continue;
            }
        };
        let days = day_str
            .chars()
            .map(|c| format!("{}", c))
            .map(|c| Day::from_str(c.as_str()))
            .collect::<Result<Vec<_>, DayParseError>>();

        let days = match days {
            Ok(days) => days,
            Err(err) => {
                eprintln!("Failed to parse days: \"{}\". Reason: {}", day_str, err);
                continue;
            }
        };

        if days.len() != 0 {
            let time = tds[9].text().next().unwrap();
            if time == "TBA" {
                continue;
            }
            // let times: Vec<&str> = time.split("-").collect::<Vec<_>>();
            // assert_eq!(2, times.len(), "Not two times (start/end) :: {:?}", times);
            // let start = parse_time(times[0]);
            // let end = parse_time(times[1]);
            let (start, end) = match try_parse_time_range(time) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Failed to parse time range: \"{}\" - {}", time, err);
                    continue;
                }
            };
            let period_type = None;
            let prof = tds[19]
                .text()
                .next()
                .unwrap()
                .replace("   ", " ")
                .replace(" (", "");
            let loc = tds[21].text().next().unwrap();
            section.periods.push(Period {
                time_start: start,
                time_end: end,
                period_type,
                location: if loc.trim().len() == 0 {
                    None
                } else {
                    Some(loc.to_string())
                },
                instructor: prof,
                days,
            })
        }
    }
    db
}


fn try_parse_time_range(s: &str) -> Result<(TimeCode, TimeCode), CourseDBHTMLParseError> {
    let spls = s.split("-").collect::<Vec<_>>();
    if spls.len() == 2 {
        try_parse_time(spls[0])
            .and_then(|time1| try_parse_time(spls[1]).map(|time2| (time1, time2)))
    } else {
        Err(CourseDBHTMLParseError::NotTwoTimes)
    }
}

fn try_parse_time(s: &str) -> Result<TimeCode, CourseDBHTMLParseError> {
    let spls = s.split(" ").collect::<Vec<_>>();
    if spls.len() == 2 {
        let pm = match spls[1] {
            "am" => Ok(false),
            "pm" => Ok(true),
            _ => Err(CourseDBHTMLParseError::MalformedTime(MalformedTimeKind::MalformedAMPM))
        };
        pm.and_then(|pm| {
            let time_cleaned = spls[0].chars().filter(|c| c.is_ascii_digit()).collect::<String>();
            let initial_timecode = u16::from_str(time_cleaned.as_str())
                .map_err(|err| CourseDBHTMLParseError::ParseIntErr(err));
            initial_timecode.map(|timecode| (pm, timecode))
        })
            .and_then(|(pm, timecode)| {
                let timecode = if pm && timecode < 1200 { timecode + 1200 } else { timecode };
                TimeCode::try_from(timecode)
                    .map_err(|err| CourseDBHTMLParseError::TimeCodeParseError(TimeCodeParseError::new(spls[0].into(), TimeCodeParseErrorKind::InvalidTime(err))))
            })
    } else {
        Err(CourseDBHTMLParseError::MalformedTime(MalformedTimeKind::MissingAMPM))
    }
}

#[derive(Debug, Clone)]
pub enum CourseDBHTMLParseError {
    NotTwoTimes,
    MalformedTime(MalformedTimeKind),
    TimeCodeParseError(TimeCodeParseError),
    ParseIntErr(ParseIntError),
}

impl Display for CourseDBHTMLParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotTwoTimes => write!(f, "Two times should be present"),
            Self::MalformedTime(mal) => write!(f, "Time is malformed: [{}]", mal),
            Self::TimeCodeParseError(err) => write!(f, "Time code failed to parse: [{}]", err),
            Self::ParseIntErr(err) => write!(f, "Integer failed to parse: [{}]", err),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MalformedTimeKind {
    MissingAMPM,
    MalformedAMPM,
}

impl Display for MalformedTimeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAMPM => write!(f, "missing AM/PM - should be \"am\" or \"pm\""),
            Self::MalformedAMPM => write!(f, "AM/PM is malformed - should be \"am\" or \"pm\""),
        }
    }
}
