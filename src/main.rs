mod actions;
mod models;
mod opt;

mod parser;

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Read};
use crate::models::*;
use crate::opt::{AppWhereIsClass, ParseArgs};
use crate::parser::{CourseDBParseError, CourseDBParser};


fn main() {
    let args = opt::parse_args();

    if let Err(err) = match args {
        AppWhereIsClass::ParseHtml(args) => {
            parse(args, parser::HtmlParser)
        }
        #[cfg(feature = "rcosxml")]
        AppWhereIsClass::ParseRcos(args) => {},
        AppWhereIsClass::FindCourseInRoom {
            db,
            room,
            day,
            time,
        } => {
            unimplemented!()
        },
        AppWhereIsClass::EmptyRooms {
            db,
            day,
            time_start,
            time_end,
        } => {
            unimplemented!()
        }
    } {
        eprintln!("An error occurred.");
        eprintln!("{}", err);

    }
}

enum ApplicationError {
    IOError(std::io::Error),
    JsonSerializationError(serde_json::Error),
    ParseError(CourseDBParseError),
    OutputExists,
    InputDoesNotExist,
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(err) => write!(f, "IO Error: {}", err),
            Self::JsonSerializationError(err) => write!(f, "Error during JSON writing: {}", err),
            Self::ParseError(err) => write!(f, "Error during parsing: {}", err),
            Self::OutputExists => write!(f, "Refusing to overwrite existing output file. Use --force to override."),
            Self::InputDoesNotExist => write!(f, "Input file does not exist.")
        }
    }
}

fn parse(parse_args: ParseArgs, parser: impl CourseDBParser) -> Result<(), ApplicationError> {
    if parse_args.file.exists() {

        if parse_args.force || !parse_args.output.exists() {
            let mut f = File::open(parse_args.file);
            f.map(|file| BufReader::new(file))
                .and_then(|mut reader| {
                    let mut content = String::with_capacity(32000);
                    reader.read_to_string(&mut content).map(|_| content)
                })
                .map_err(|err| ApplicationError::IOError(err))
                .and_then(|content| parser.parse(content.as_str()).map_err(|err| ApplicationError::ParseError(err)))
                .and_then(|db| File::create(parse_args.output.as_path()).map_err(|err| ApplicationError::IOError(err)).map(|file| (db, file)))
                .and_then(|(db, file)| serde_json::to_writer(file, &db).map_err(|err| ApplicationError::JsonSerializationError(err)))
        } else {
            Err(ApplicationError::OutputExists)
        }
    } else {
        Err(ApplicationError::InputDoesNotExist)
    }
}

// fn load_db(db_file: &str) -> CourseDB {
//     let f = File::open(db_file).expect("Opening file");
//     serde_json::from_reader(f).expect("Parsing CourseDB")
// }
//
// fn empty_rooms(db_file: &str, time_start: &str, time_end: &str, day: &str) {
//     let db = load_db(db_file);
//     let empty = db.find_empty_rooms(time_start.parse().unwrap(), time_end.parse().unwrap(), Day::from(day));
//     println!("{} empty room{} found between {} and {}:\n", empty.len(), if empty.len() != 1 { "s" } else {""}, time_start, time_end );
//     for room in empty {
//         println!("{}", room);
//     }
// }
//
// fn find_course_in_room(db_file: &str, room: &str, time: &str, day: &str) {
//     println!("{} -- ", room);
//     let courses = load_db(db_file).find_course_in_room_at_time(room, time.parse().unwrap(), Day::from(day));
//     println!("Found the following course{}:", if courses.len() != 1 { "s" } else {""});
//     for course in courses {
//         println!("{} {} -- {}", course.dept, course.num, course.name);
//     }
// }
//
// fn parsehtml(input: PathBuf, output: PathBuf) {
//     std_parse(input, output, |s| htmlparser::parse_html(s))
// }
//
// fn parsercos(input: PathBuf, output: PathBuf) {
//     std_parse(input, output, |input| xml_parser::parse_db(input, false).expect("Parsing xml db"))
// }
//
// fn std_parse<F: FnOnce(&str) -> CourseDB>(input: PathBuf, output: PathBuf, parser: F) {
//     let mut f = File::open(input).expect("Opening file");
//     let mut fragment = String::new();
//     f.read_to_string(&mut fragment).expect("Reading file");
//     let db = parser(fragment.as_ref());
//     println!("Read {} courses", db.courses.len());
//     serde_json::to_writer_pretty(File::create(output).expect("Creating output file"), &db).expect("Writing output file");
// }
//
// fn verify_exists(input: String) -> Result<(), String> {
//     if Path::new(input.as_str()).is_file() { Ok(()) }
//     else { Err(String::from("File does not exist!")) }
// }
