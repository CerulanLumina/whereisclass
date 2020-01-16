use crate::{Deserialize, Serialize};

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
    pub time_start: u16,
    /// The ending time. See [`time_start`](Period::time_start) for information on format.
    pub time_end: u16,
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

impl<S: AsRef<str>> From<S> for Day {
    fn from(from: S) -> Self {
        match from.as_ref() {
            "0" => Self::Monday,
            "1" => Self::Tuesday,
            "2" => Self::Wednesday,
            "3" => Self::Thursday,
            "4" => Self::Friday,
            "M" => Self::Monday,
            "T" => Self::Tuesday,
            "W" => Self::Wednesday,
            "R" => Self::Thursday,
            "F" => Self::Friday,
            x => panic!("Unknown day '{}'!!", x)
        }
    }
}
