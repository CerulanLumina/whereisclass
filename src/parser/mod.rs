use std::fmt::{Display, Formatter};
use crate::CourseDB;
use crate::parser::htmlparser::CourseDBHTMLParseError;

mod htmlparser;

pub use htmlparser::HtmlParser;

#[cfg(feature = "rcosxml")]
mod xml_parser;

pub trait CourseDBParser {
    fn parse(&self, input: &str) -> Result<CourseDB, CourseDBParseError>;
}

pub enum CourseDBParseError {

}

impl Display for CourseDBParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
