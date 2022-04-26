//! A monolithic block of code that handles parsing of a ROCS xml file.

use crate::{models, TimeCode};
use roxmltree::{Document, Node};
use std::{num::ParseIntError, str::FromStr};
use whereisclass::models;

fn parse_day(day: Node) -> Result<models::Day, CourseDBError> {
    models::Day::from_str(day.text()?).map_err(|a| a.into())
}

fn parse_period(period_node: Node, strict: bool) -> Result<models::Period, CourseDBError> {
    let time_regex = regex::Regex::new(r#"^\d+$"#).unwrap();
    let time_start = period_node.attribute("start")?;
    let time_end = period_node.attribute("end")?;

    if !time_regex.is_match(time_start) || !time_regex.is_match(time_end) {
        return Err(CourseDBError::MissingValue);
    }

    let time_start = TimeCode::from_str(time_start)?;
    let time_end = TimeCode::from_str(time_end)?;

    let location = period_node.attribute("location").map(|s| s.to_string());
    let period_type = period_node.attribute("type").map(|s| s.into());

    let instructor = period_node.attribute("instructor")?.to_string();

    let mut period = models::Period {
        time_start,
        time_end,
        instructor,
        location,
        period_type,
        days: Vec::new(),
    };

    for day in period_node
        .children()
        .filter(|child| child.tag_name().name() == "DAY")
    {
        let res = parse_day(day);
        if res.is_err() {
            let err = res.unwrap_err();
            if VERBOSE {
                match err {
                    CourseDBError::ParsingNum => eprintln!(
                        "Failed to parse day due to malformed number, XML: {:?}",
                        day
                    ),
                    CourseDBError::MissingValue => {
                        eprintln!("Failed to parse day due to empty value, XML: {:?}", day)
                    }
                }
            }
            if strict {
                return Err(err);
            }
        } else {
            period.days.push(res.unwrap());
        }
    }
    Ok(period)
}

fn parse_note(note: Node) -> Result<String, CourseDBError> {
    let note = note.text()?.to_string();
    Ok(note)
}

fn parse_section(section_node: Node, strict: bool) -> Result<models::Section, CourseDBError> {
    let crn = section_node.attribute("crn")?;
    let crn = u32::from_str(crn)?;
    let num = section_node.attribute("num")?;
    let num = u8::from_str(num)?;

    let mut section = models::Section {
        crn,
        num,
        periods: Vec::new(),
        notes: Vec::new(),
    };

    for period in section_node
        .children()
        .filter(|child| child.tag_name().name() == "PERIOD")
    {
        let res = parse_period(period, strict);
        if res.is_err() {
            let err = res.unwrap_err();
            if VERBOSE {
                match err {
                    CourseDBError::ParsingNum => eprintln!(
                        "Failed to parse period due to malformed number, XML: {:?}",
                        period
                    ),
                    CourseDBError::MissingValue => eprintln!(
                        "Failed to parse period due to empty value, XML: {:?}",
                        period
                    ),
                }
            }
            if strict {
                return Err(err);
            }
        } else {
            section.periods.push(res.unwrap());
        }
    }

    for note in section_node
        .children()
        .filter(|course_child| course_child.tag_name().name() == "NOTE")
    {
        let res = parse_note(note);
        if res.is_err() {
            let err = res.unwrap_err();
            match err {
                CourseDBError::ParsingNum => eprintln!(
                    "Failed to parse note due to malformed number, XML: {:?}",
                    note
                ),
                CourseDBError::MissingValue => {
                    eprintln!("Failed to parse note due to empty value, XML: {:?}", note)
                }
            }
            if strict {
                return Err(err);
            }
        } else {
            section.notes.push(res.unwrap());
        }
    }

    Ok(section)
}

fn parse_course(course_node: Node, strict: bool) -> Result<models::Course, CourseDBError> {
    let name = course_node.attribute("name")?.to_string();
    let dept = course_node.attribute("dept")?.to_string();
    let num = course_node.attribute("num")?;
    let num = u16::from_str(num)?;

    let mut course = models::Course {
        name,
        dept,
        num,
        sections: Vec::new(),
    };

    for section in course_node
        .children()
        .filter(|course_child| course_child.tag_name().name() == "SECTION")
    {
        let res = parse_section(section, strict);
        if res.is_err() {
            let err = res.unwrap_err();
            if VERBOSE {
                match err {
                    CourseDBError::ParsingNum => eprintln!(
                        "Failed to parse section due to malformed number, XML: {:?}",
                        section
                    ),
                    CourseDBError::MissingValue => eprintln!(
                        "Failed to parse section due to empty value, XML: {:?}",
                        section
                    ),
                }
            }
            if strict {
                return Err(err);
            }
        } else {
            course.sections.push(res.unwrap())
        }
    }

    Ok(course)
}

pub fn parse_db<S: AsRef<str>>(xml: S, strict: bool) -> Result<models::CourseDB, CourseDBError> {
    let mut courses = Vec::<models::Course>::new();

    let doc = Document::parse(xml.as_ref()).expect("Parsing XML");
    for course in doc
        .root_element()
        .children()
        .filter(|node| node.tag_name().name() == "COURSE")
    {
        let res = parse_course(course, strict);
        if res.is_err() {
            let err = res.unwrap_err();
            if VERBOSE {
                match err {
                    CourseDBError::ParsingNum => eprintln!(
                        "Failed to parse course due to malformed number, XML: {:?}",
                        course
                    ),
                    CourseDBError::MissingValue => eprintln!(
                        "Failed to parse course due to empty value, XML: {:?}",
                        course
                    ),
                }
            }
            return Err(err);
        } else {
            courses.push(res.unwrap());
        }
    }
    Ok(models::CourseDB { courses })
}

#[derive(Debug, Copy, Clone)]
pub enum CourseDBError {
    ParsingNum,
    MissingValue,
}

impl std::error::Error for CourseDBError {}

impl std::fmt::Display for CourseDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:?}", self) }
}

impl From<ParseIntError> for CourseDBError {
    fn from(_: ParseIntError) -> Self { Self::ParsingNum }
}

impl From<std::option::NoneError> for CourseDBError {
    fn from(_: std::option::NoneError) -> Self { Self::MissingValue }
}
