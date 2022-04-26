use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter},
    num::ParseIntError,
    ops::RangeInclusive,
    str::FromStr,
};

const VALID_TIME_RANGE: RangeInclusive<u16> = 700..=2350;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// A (de)serializable database of courses. Literally just wraps a Vec<Course>
pub struct CourseDB {
    /// The courses in the database
    pub courses: Vec<Course>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// A (de)serializable Course structure, containing a name, department code, course number,
/// and the sections.
pub struct Course {
    /// The course name / title
    pub name: String,

    /// The department the course is in (e.g. CSCI or ITWS)
    pub dept: String,

    /// The number of the course (e.g. ITWS **__4500__**)
    pub num: u16,

    /// The sections this course has.
    pub sections: Vec<Section>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// A (de)serializable section structure, containing info pertinent to sections.
/// This includes information like the CRN, the section number, and the class periods
/// the section meets, along with any notes.
pub struct Section {
    /// RPI Course registration number
    pub crn: u32,

    /// The index of the section
    pub num: u8,

    /// Which periods the section meets
    pub periods: Vec<Period>,

    /// Notes applicable to this section in `String` format.
    pub notes: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// A (de)serializable period structure that defines when a section meets
pub struct Period {
    /// The starting time - This should be in 24h military time `hhmm`.
    /// For example, 1:35 PM --> `1335`
    pub time_start: TimeCode,

    /// The ending time. See [`time_start`](Period::time_start) for information on format.
    pub time_end: TimeCode,

    /// The name of the instructor of this session. For now only holds the primary instructor
    pub instructor: String,

    /// The list of days this period is held on
    pub days: Vec<Day>,

    /// The location of this period.
    ///
    /// Optional, `Some(...)` if a location is available, `None` otherwise.
    pub location: Option<String>,

    /// The type of period this is, such as lecture, test, lab, etc...
    ///
    /// Optional, `Some(...)` if the period type is known, otherwise `None`.
    pub period_type: Option<PeriodType>,
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Ord, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct TimeCode {
    time: u16,
}

impl TimeCode {
    pub fn time(&self) -> u16 { self.time }
    pub unsafe fn new_from_int(time_code: u16) -> Self { Self { time: time_code } }
}

impl Display for TimeCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.time / 100, self.time % 100)
    }
}

impl Into<u16> for TimeCode {
    fn into(self) -> u16 { self.time }
}

impl TryFrom<u16> for TimeCode {
    type Error = TimeCodeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if VALID_TIME_RANGE.contains(&value) {
            if value % 100 < 60 {
                // Safety: checked at this point to be valid
                Ok(unsafe { Self::new_from_int(value) })
            } else {
                Err(TimeCodeError::InvalidMinutes)
            }
        } else {
            Err(TimeCodeError::OutOfBounds)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TimeCodeError {
    OutOfBounds,
    InvalidMinutes,
}

impl std::error::Error for TimeCodeError {}

impl Display for TimeCodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds => write!(
                f,
                "Time must be between {} and {} (inclusive).",
                VALID_TIME_RANGE.start(),
                VALID_TIME_RANGE.end()
            ),
            Self::InvalidMinutes => write!(f, "Minutes of time code exceeds 59."),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, PartialOrd, Serialize, Deserialize)]
/// A (de)serializable enum that represents a weekday
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
/// A (de)serializable enum that represents a period type
pub enum PeriodType {
    Lecture,
    Recitation,
    Lab,
    Test,
    Other(String),
}

impl<S: AsRef<str>> From<S> for PeriodType {
    fn from(from: S) -> Self {
        match from.as_ref() {
            "LEC" => Self::Lecture,
            "REC" => Self::Recitation,
            "LAB" => Self::Lab,
            "TST" => Self::Test,
            other => Self::Other(other.to_string()),
        }
    }
}

impl FromStr for Day {
    type Err = DayParseError;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        match from {
            "0" => Ok(Self::Monday),
            "1" => Ok(Self::Tuesday),
            "2" => Ok(Self::Wednesday),
            "3" => Ok(Self::Thursday),
            "4" => Ok(Self::Friday),
            "M" => Ok(Self::Monday),
            "T" => Ok(Self::Tuesday),
            "W" => Ok(Self::Wednesday),
            "R" => Ok(Self::Thursday),
            "F" => Ok(Self::Friday),
            x => Err(DayParseError(from.into())),
        }
    }
}

pub struct DayParseError(String);

impl Debug for DayParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Input must match /^[01234MTWRF]$/. Provided: {:?}",
            self.0
        )
    }
}

impl Display for DayParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Input (\"{}\") must be one of: [M, T, W, R, F] or as digits: [0, 1, 2, 3, 4]",
            self.0
        )
    }
}

impl std::error::Error for DayParseError {}

impl FromStr for TimeCode {
    type Err = TimeCodeParseError;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        u16::from_str(from)
            .map_err(|err| {
                TimeCodeParseError::new(from.into(), TimeCodeParseErrorKind::ParseIntError(err))
            })
            .and_then(|num| {
                TimeCode::try_from(num).map_err(|a| {
                    TimeCodeParseError::new(from.into(), TimeCodeParseErrorKind::InvalidTime(a))
                })
            })
    }
}

#[derive(Debug, Clone)]
pub struct TimeCodeParseError {
    input: String,
    kind: TimeCodeParseErrorKind,
}

impl Display for TimeCodeParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error parsing time code. Provided: {}. Reason: {}",
            self.input,
            self.kind()
        )
    }
}

impl TimeCodeParseError {
    pub fn new(input: String, kind: TimeCodeParseErrorKind) -> TimeCodeParseError {
        Self { input, kind }
    }

    fn kind(&self) -> &TimeCodeParseErrorKind { &self.kind }

    fn input(&self) -> &str { self.input.as_str() }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TimeCodeParseErrorKind {
    InvalidTime(TimeCodeError),
    ParseIntError(ParseIntError),
}

impl Display for TimeCodeParseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseIntError(err) => write!(f, "Parsing integer: {}", err),
            Self::InvalidTime(err) => write!(f, "Invalid time: {}", err),
        }
    }
}

impl std::error::Error for TimeCodeParseError {}
