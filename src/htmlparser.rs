use crate::models::*;
use scraper::{Html, Selector};
use std::str::FromStr;

pub fn parse_html<S: AsRef<str>>(input: S) -> CourseDB {

    let mut db = CourseDB { courses: Vec::new() };

    let fragment = input.as_ref().replace("\n", "");

    let html = Html::parse_fragment(fragment.as_str());
    let trsel = Selector::parse("tr").unwrap();
    let tdsel = Selector::parse("td").unwrap();

    let mut last_course = 0usize;

    let day_regex = regex::Regex::new(r"^[MTWRF]*$").expect("Invalid regex");

    for tr in html.select(&trsel) {
        let tds = tr.select(&tdsel).collect::<Vec<_>>();
        if tds.len() == 0 {continue;}
        let section_str = tds[4].text().next().unwrap_or("00");
        if section_str == "H01" { continue; }
        let course = if section_str == "01" {
            // new course
            db.courses.push(Course {
                name: tds[7].text().next().unwrap().to_string(),
                dept: tds[2].text().next().unwrap().to_string(),
                num: u16::from_str(tds[3].text().next().unwrap()).unwrap(),
                sections: vec![]
            });
            last_course = db.courses.len() - 1;
            db.courses.get_mut(last_course).unwrap()
        } else {
            db.courses.get_mut(last_course).unwrap()
        };
        let section_len = course.sections.len();
        let is_period = !section_str.as_bytes()[0].is_ascii_digit();
        let section = if is_period { course.sections.get_mut(section_len - 1).unwrap() } else {
            course.sections.push(Section {
                crn: u32::from_str(tds[1].text().next().unwrap()).expect("Reading u32 for crn"),
                num: u8::from_str(section_str).expect("Reading section"),
                notes: vec![],
                periods: vec![]
            });
            course.sections.get_mut(section_len).unwrap()
        };

        let day_str_opt = tds[8].text().filter(|k| k != &"TBA" && day_regex.is_match(k)).next();
        if day_str_opt.is_none() { continue; }
        let days = day_str_opt.unwrap().chars().map(|c| format!("{}", c)).map(Day::from).collect::<Vec<_>>();

        if days.len() != 0 {
            let time = tds[9].text().next().unwrap();
            if time == "TBA" { continue; }
            let times: Vec<&str> = time.split("-").collect::<Vec<_>>();
            assert_eq!(2, times.len(), "Not two times (start/end) :: {:?}", times);
            let start = parse_time(times[0]);
            let end = parse_time(times[1]);
            let period_type = None;
            let prof = tds[19].text().next().unwrap().replace("   ", " ").replace(" (", "");
            let loc = tds[21].text().next().unwrap();
            section.periods.push(Period { time_start: start, time_end: end, period_type, location: if loc.trim().len() == 0 {None} else {Some(loc.to_string())}, instructor: prof, days })
        }


    }
    db
}

fn parse_time(s: &str) -> u16 {
    let pm = s.contains("pm");
    let pm_offset: u16 = if pm { 1200 } else { 0 };
    u16::from_str(s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().as_ref()).expect("Couldn't parse time") + pm_offset
}