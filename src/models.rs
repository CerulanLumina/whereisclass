use crate::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CourseDB {
    pub courses: Vec<Course>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Course {
    pub name: String,
    pub dept: String,
    pub num: u16,
    pub sections: Vec<Section>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Section {
    pub crn: u32,
    pub num: u8,
    pub periods: Vec<Period>,
    pub notes: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Period {
    pub time_start: String,
    pub time_end: String,
    pub days: Vec<Day>,
    pub location: Option<String>,
    pub period_type: Option<PeriodType>,
}

#[derive(Clone, PartialEq, Debug, PartialOrd, Serialize, Deserialize)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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
            _ => panic!("Unknown day!!")
        }
    }
}
