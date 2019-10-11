#![feature(try_trait)]

use roxmltree::{Document, Node};
use std::io::Read;
use std::fs::File;

mod models;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::models::CourseDB;
use std::num::ParseIntError;

const VERBOSE: bool = false;

fn parse_days(day: Node, day_vec: &mut Vec<models::Day>) -> Result<(), CourseDBError> {
    let day = day.text()?.into();
    day_vec.push(day);
    Ok(())
}

fn parse_periods(period: Node, period_vec: &mut Vec<models::Period>) -> Result<(), CourseDBError> {
    let time_regex = regex::Regex::new(r#"^\d+$"#).unwrap();
    let time_start = period.attribute("start")?.to_string();
    let time_end = period.attribute("end")?.to_string();

    if !time_regex.is_match(time_start.as_str()) || !time_regex.is_match(time_end.as_str()) {
        return Err(CourseDBError::MissingValue)
    }

    let location = period.attribute("location").map(|s| s.to_string());
    let period_type = period.attribute("type").map(|s| s.into());
    let mut days = Vec::new();
    period.children()
        .filter(|child| child.tag_name().name() == "DAY")
        .for_each(|day| {
            let res = parse_days(day, &mut days);
            if res.is_err() && VERBOSE {
                match res.unwrap_err() {
                    CourseDBError::ParsingNum => eprintln!("Failed to parse day due to malformed number, XML: {:?}", day),
                    CourseDBError::MissingValue => eprintln!("Failed to parse day due to empty value, XML: {:?}", day),
                }
            }
        });

    period_vec.push(models::Period {
        time_start,
        time_end,
        location,
        period_type,
        days
    });
    Ok(())

}

fn parse_notes(note: Node, note_vec: &mut Vec<String>) -> Result<(), CourseDBError> {
    note_vec.push(note.text()?.to_string());
    Ok(())
}

fn parse_section(section: Node, section_vec: &mut Vec<models::Section>) -> Result<(), CourseDBError> {
    let crn = section.attribute("crn")?;
    let crn = u32::from_str(crn)?;
    let num = section.attribute("num")?;
    let num = u8::from_str(num)?;
    let mut periods = Vec::new();
    let mut notes = Vec::new();
    section.children()
        .filter(|child| child.tag_name().name() == "PERIOD")
        .for_each(|period| {
            let res = parse_periods(period, &mut periods);
            if res.is_err() && VERBOSE {
                match res.unwrap_err() {
                    CourseDBError::ParsingNum => eprintln!("Failed to parse period due to malformed number, XML: {:?}", period),
                    CourseDBError::MissingValue => eprintln!("Failed to parse period due to empty value, XML: {:?}", period),
                }
            }
        });
    section.children()
        .filter(|course_child| course_child.tag_name().name() == "NOTE")
        .for_each(|note| {
            let res = parse_notes(note, &mut notes);
            if res.is_err() && VERBOSE {
                match res.unwrap_err() {
                    CourseDBError::ParsingNum => eprintln!("Failed to parse note due to malformed number, XML: {:?}", note),
                    CourseDBError::MissingValue => eprintln!("Failed to parse note due to empty value, XML: {:?}", note),
                }
            }
        });

    section_vec.push(models::Section {
        crn,
        num,
        periods,
        notes,
    });

    Ok(())

}

fn parse_course(course: Node, course_vec: &mut Vec<models::Course>) -> Result<(), CourseDBError> {
    let name = course.attribute("name").expect("No name course").to_string();
    let dept = course.attribute("dept").expect("No dept course").to_string();
    let num = course.attribute("num").expect("no num course");
    let num = u16::from_str(num).expect("Converting num to u16");

    let mut sections = Vec::<models::Section>::new();

    course.children()
        .filter(|course_child| course_child.tag_name().name() == "SECTION")
        .for_each(|section| {
            let res = parse_section(section, &mut sections);
            if res.is_err() && VERBOSE {
                match res.unwrap_err() {
                    CourseDBError::ParsingNum => eprintln!("Failed to parse section due to malformed number, XML: {:?}", section),
                    CourseDBError::MissingValue => eprintln!("Failed to parse section due to empty value, XML: {:?}", section),
                }
            }
        });

    course_vec.push(models::Course {
        name,
        dept,
        num,
        sections,
    });

    Ok(())


}

fn res(s: Option<i32>) -> Result<(), CourseDBError> {
    s?;
    Ok(())
}

fn main() {
    let text = {
        let mut file = File::open("201909.xml").expect("Opening file");
        let mut text = String::new();
        file.read_to_string(&mut text).expect("Reading file");
        text
    };

    let mut coursevec = Vec::<models::Course>::new();

    let doc = Document::parse(text.as_str()).expect("Parsing XML");
    doc.root_element()
        .children()
        .filter(|node| node.tag_name().name() == "COURSE")
        .for_each(|course| {
            let res = parse_course(course, &mut coursevec);
            if res.is_err() && VERBOSE {
                match res.unwrap_err() {
                    CourseDBError::ParsingNum => eprintln!("Failed to parse course due to malformed number, XML: {:?}", course),
                    CourseDBError::MissingValue => eprintln!("Failed to parse course due to empty value, XML: {:?}", course),
                }
            }
        });

    let db = CourseDB { courses: coursevec };

    db.courses.iter().filter(|course| course.sections.iter().any(|section| section.periods.iter().any(|period| period.time_start == "1000" && period.location == Some("SAGE 2715".into()) && period.days.contains(&models::Day::Tuesday))))
        .for_each(|course| {
            println!("{} - {} {}", course.name, course.dept, course.num);
            course.sections.iter()
                .filter(|section| section.periods.iter().any(|period| period.time_start == "1000" && period.location == Some("SAGE 2715".into()) && period.days.contains(&models::Day::Tuesday)))
                .for_each(|section| {
                    println!("\t[{}] ({})", section.num, section.crn);
                    section.periods.iter()
                        .filter(|period| period.time_start == "1000" && period.location == Some("SAGE 2715".into()) && period.days.contains(&models::Day::Tuesday))
                        .for_each(|period| {
                            println!("\t\t {} - {} @ {} on {:?}", period.time_start, period.time_end, period.location.as_ref().unwrap(), period.days);
                        });
                });
        });

    res(Some(3)).unwrap();

//    println!("Hello, world!");
}

#[derive(Debug)]
enum CourseDBError {
    ParsingNum,
    MissingValue,
}

impl std::error::Error for CourseDBError {}
impl std::fmt::Display for CourseDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseIntError> for CourseDBError {
    fn from(_: ParseIntError) -> Self {
        Self::ParsingNum
    }
}

impl From<std::option::NoneError> for CourseDBError {
    fn from(_: std::option::NoneError) -> Self {
        Self::MissingValue
    }
}

