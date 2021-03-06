#![feature(try_trait)]
#![deny(unused_must_use)]

extern crate postgres;
extern crate roxmltree;
extern crate scraper;
extern crate inflector;
extern crate bimap;
extern crate dotenv;

use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::str::FromStr;

use clap::{Arg, SubCommand};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::actions::{FindCourseInRoomAtTime, FindEmptyRooms};
use crate::models::*;
use postgres::Config;

mod models;
mod xml_parser;
mod htmlparser;
mod actions;
mod pgdb;

fn main() {
    dotenv::dotenv().ok();

    let app = clap::App::new("whereisclass");

    let app = app
        .version("0.1.0")
        .about("A toolkit to find out nifty information about the RPI master schedule.")
        .subcommand(SubCommand::with_name("parsehtml").about("Parse an HTML file containing just a table of all classes formatted like SIS into JSON.")
            .arg(Arg::with_name("file").required(true).help("Sets the html file to read from, following the SIS Table format").validator(verify_exists))
            .arg(Arg::with_name("output").required(true).help("Sets the output JSON file")))
        .subcommand(SubCommand::with_name("parsercos").about("Parse an RCOS XML file into JSON")
            .arg(Arg::with_name("file").required(true).help("Sets the html file to read from, following the SIS Table format").validator(verify_exists))
            .arg(Arg::with_name("output").required(true).help("Sets the output JSON file")))
        .subcommand(SubCommand::with_name("find-course-in-room").about("Determines which courses are being held in a given room at a given time")
            .arg(Arg::with_name("db").required(true).help("The JSON Course DB to scan").validator(verify_exists))
            .arg(Arg::with_name("room").required(true).help("The SIS room name"))
            .arg(Arg::with_name("time").required(true).help("The military time code").validator(verify_number))
            .arg(Arg::with_name("day").required(true).help("The day").validator(verify_day)))
        .subcommand(SubCommand::with_name("empty-rooms").about("Find empty rooms for a given time range")
            .arg(Arg::with_name("db").required(true).help("The JSON Course DB to scan").validator(verify_exists))
            .arg(Arg::with_name("time-start").required(true).help("The start time").validator(verify_number))
            .arg(Arg::with_name("time-end").required(true).help("The end time").validator(verify_number))
            .arg(Arg::with_name("day").required(true).help("The day").validator(verify_day)))
        .subcommand(SubCommand::with_name("dbpush").about("Push selected course db to database")
            .arg(Arg::with_name("db").required(true).help("The JSON Course DB to scan").validator(verify_exists))
            .arg(Arg::with_name("dbhost").required_unless("connection").help("The Postgres host to use").validator(valid_inet).env("DB_HOST"))
            .arg(Arg::with_name("dbuser").required_unless("connection").help("The PG username to use").env("DB_USER"))
            .arg(Arg::with_name("dbpass").required_unless("connection").help("The PG password to use").env("DB_PASS"))
            .arg(Arg::with_name("dbname").required_unless("connection").help("Which database to insert into").env("DB_NAME"))
            .arg(Arg::with_name("connection").required_unless_all(&["dbhost", "dbuser", "dbpass", "dbname"]).help("The connection string to use").env("DB_CONN").validator(valid_conn))
        );

    let matches = app.get_matches();
    let (subcommand, subc_matches) = matches.subcommand();

    let valid = match subcommand {
        "parsehtml" |
        "find-course-in-room" |
        "empty-rooms" |
        "dbpush" |
        "parsercos" => true,
        _ => false
    };

    if !valid {
        eprintln!("Invalid subcommand.");
        std::process::exit(1);
    }

    let subc_matches = subc_matches.unwrap();

    match subcommand {
        "parsehtml" => parsehtml(subc_matches.value_of("file").unwrap(), subc_matches.value_of("output").unwrap()),
        "parsercos" => parsercos(subc_matches.value_of("file").unwrap(), subc_matches.value_of("output").unwrap()),
        "find-course-in-room" => find_course_in_room(subc_matches.value_of("db").unwrap(),
                                                     subc_matches.value_of("room").unwrap(),
                                                     subc_matches.value_of("time").unwrap(),
                                                     subc_matches.value_of("day").unwrap()),
        "empty-rooms" => empty_rooms(subc_matches.value_of("db").unwrap(),
                                     subc_matches.value_of("time-start").unwrap(),
                                     subc_matches.value_of("time-end").unwrap(),
                                     subc_matches.value_of("day").unwrap()),
        "dbpush" => {
            if subc_matches.is_present("connection") {
                dbpush_conn(subc_matches.value_of("db").unwrap(),
                            subc_matches.value_of("connection").unwrap())
            } else {
                dbpush_args(subc_matches.value_of("db").unwrap(),
                            subc_matches.value_of("dbhost").unwrap(),
                            subc_matches.value_of("dbuser").unwrap(),
                            subc_matches.value_of("dbpass").unwrap(),
                            subc_matches.value_of("dbname").unwrap())
            }
        }
        _ => panic!("Unknown command"),
    }
}

fn dbpush_args(db_file: &str, dbhost: &str, dbuser: &str, dbpass: &str, dbname: &str) {
    let db = load_db(db_file);
    let dbhost = IpAddr::from_str(dbhost).map(|ipa| SocketAddr::new(ipa, 5432)).map_err(|_| ())
        .or(SocketAddr::from_str(dbhost).map_err(|_| ())).expect("parsing addr");

    let config_str = format!("user={} password={} dbname={} host={} port={}", dbuser, dbpass, dbname, dbhost.ip().to_string(), dbhost.port());
    let res = Config::from_str(config_str.as_str());
    if let Ok(conf) = res {
        dbpush(db, conf);
    } else if let Err(err) = res {
        eprintln!("Failed to setup connection parameters due to weird inputs.");
        eprintln!("{}", err);
    }
}

fn dbpush_conn(db_file: &str, dbconn: &str) {
    let db = load_db(db_file);
    let config = Config::from_str(dbconn).expect("dbconn parse");
    dbpush(db, config);
}

fn dbpush(db: CourseDB, con: Config) {
    println!("Pushing loaded database to Postgres...");
    let res = pgdb::push_db(db, con);
    if let Err(err) = res {
        eprintln!("Failed to push database.");
        eprintln!("Reason: {}", err);
    } else {
        println!("Completed.");
    }
}


fn valid_conn(input: String) -> Result<(), String> {
    Config::from_str(input.as_str()).map_err(|e| e.to_string()).map(|_| ())
}

fn valid_inet(input: String) -> Result<(), String> {
    IpAddr::from_str(input.as_str()).map(|_| ()).map_err(|e| e.to_string())
        .or(SocketAddr::from_str(input.as_str()).map(|_| ()).map_err(|e| e.to_string()))
}

fn load_db(db_file: &str) -> CourseDB {
    let f = File::open(db_file).expect("Opening file");
    serde_json::from_reader(f).expect("Parsing CourseDB")
}

fn empty_rooms(db_file: &str, time_start: &str, time_end: &str, day: &str) {
    let db = load_db(db_file);
    let empty = db.find_empty_rooms(time_start.parse().unwrap(), time_end.parse().unwrap(), Day::from(day));
    println!("{} empty room{} found between {} and {}:\n", empty.len(), if empty.len() != 1 { "s" } else { "" }, time_start, time_end);
    for room in empty {
        println!("{}", room);
    }
}

fn find_course_in_room(db_file: &str, room: &str, time: &str, day: &str) {
    println!("{} -- ", room);
    let courses = load_db(db_file).find_course_in_room_at_time(room, time.parse().unwrap(), Day::from(day));
    println!("Found the following course{}:", if courses.len() != 1 { "s" } else { "" });
    for course in courses {
        println!("{} {} -- {}", course.dept, course.num, course.name);
    }
}

fn parsehtml(input: &str, output: &str) {
    std_parse(input, output, |s| htmlparser::parse_html(s))
}

fn parsercos(input: &str, output: &str) {
    std_parse(input, output, |input| xml_parser::parse_db(input, false).expect("Parsing xml db"))
}

fn std_parse<F: FnOnce(&str) -> CourseDB>(input: &str, output: &str, parser: F) {
    let mut f = File::open(input).expect("Opening file");
    let mut fragment = String::new();
    f.read_to_string(&mut fragment).expect("Reading file");
    let db = parser(fragment.as_ref());
    println!("Read {} courses", db.courses.len());
    serde_json::to_writer_pretty(File::create(output).expect("Creating output file"), &db).expect("Writing output file");
}

fn verify_exists(input: String) -> Result<(), String> {
    if Path::new(input.as_str()).is_file() { Ok(()) } else { Err(String::from("File does not exist!")) }
}

fn verify_number(input: String) -> Result<(), String> {
    u32::from_str(input.as_str()).map(|_| ()).map_err(|_| String::from("Unable to parse integer"))
}

fn verify_day(input: String) -> Result<(), String> {
    let regex = Regex::new(r"^[01234MTWRF]$").unwrap();
    if regex.is_match(input.as_str()) { Ok(()) } else { Err(String::from("Must be one of: M, T, W, R, F, 0, 1, 2, 3, 4")) }
}
