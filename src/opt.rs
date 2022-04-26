// let app = clap::App::new("whereisclass");
//
// let app = app
// .version("0.1.0")
// .about("A toolkit to find out nifty information about the RPI master schedule.")
// .subcommand(SubCommand::with_name("parsehtml").about("Parse an HTML file containing just a table of all classes formatted like SIS into JSON.")
// .arg(Arg::with_name("file").required(true).help("Sets the html file to read from, following the SIS Table format").validator(verify_exists))
// .arg(Arg::with_name("output").required(true).help("Sets the output JSON file")))
// .subcommand(SubCommand::with_name("parsercos").about("Parse an RCOS XML file into JSON")
// .arg(Arg::with_name("file").required(true).help("Sets the html file to read from, following the SIS Table format").validator(verify_exists))
// .arg(Arg::with_name("output").required(true).help("Sets the output JSON file")))
// .subcommand(SubCommand::with_name("find-course-in-room").about("Determines which courses are being held in a given room at a given time")
// .arg(Arg::with_name("db").required(true).help("The JSON Course DB to scan").validator(verify_exists))
// .arg(Arg::with_name("room").required(true).help("The SIS room name"))
// .arg(Arg::with_name("time").required(true).help("The military time code").validator(verify_number))
// .arg(Arg::with_name("day").required(true).help("The day").validator(verify_day)))
// .subcommand(SubCommand::with_name("empty-rooms").about("Find empty rooms for a given time range")
// .arg(Arg::with_name("db").required(true).help("The JSON Course DB to scan").validator(verify_exists))
// .arg(Arg::with_name("time-start").required(true).help("The start time").validator(verify_number))
// .arg(Arg::with_name("time-end").required(true).help("The end time").validator(verify_number))
// .arg(Arg::with_name("day").required(true).help("The day").validator(verify_day))
// );
//

use crate::models::{Day, TimeCode};
use std::path::PathBuf;
use structopt::StructOpt;

pub fn parse_args() -> AppWhereIsClass {
    AppWhereIsClass::from_args()
}

#[derive(StructOpt, Debug)]
#[structopt(name = "whereisclass")]
pub enum AppWhereIsClass {
    ParseHtml(ParseArgs),
    #[cfg(feature = "rcosxml")]
    ParseRcos(ParseArgs),
    FindCourseInRoom {
        db: PathBuf,
        room: String,
        time: TimeCode,
        day: Day,
    },
    EmptyRooms {
        db: PathBuf,
        time_start: TimeCode,
        time_end: TimeCode,
        day: Day,
    },
}

#[derive(StructOpt, Debug, Clone)]
pub struct ParseArgs {
    /// Forcibly overwrite the output file
    #[structopt(short, long)]
    pub force: bool,

    /// Input file to parse
    pub file: PathBuf,

    /// Output file to write, will not overwrite unless --force is specified
    pub output: PathBuf,
}
