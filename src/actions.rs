use crate::models;
use std::collections::HashSet;

pub trait FindCourseInRoomAtTime {
    fn find_course_in_room_at_time(&self, room: &str, time: u16, day: models::Day) -> Vec<models::Course> {
        self.find_course_in_room_at_time_range(room, time, time, day)
    }
    fn find_course_in_room_at_time_range(&self, room: &str, time_start: u16, time_end: u16, day: models::Day) -> Vec<models::Course>;
}

pub trait FindEmptyRooms {
    fn find_empty_rooms(&self, time_start: u16, time_end: u16, day: models::Day) -> Vec<String>;
}

impl FindCourseInRoomAtTime for models::CourseDB {
    fn find_course_in_room_at_time_range(&self, room: &str, time_start: u16, time_end: u16, day: models::Day) -> Vec<models::Course> {
        let mut clash = Vec::new();
        for course in &self.courses {
            for section in &course.sections {
                for period in &section.periods {
                    if period.location.is_some() {
                        let loc = period.location.as_ref().unwrap();
                        let time_start_between = period.time_start <= time_start && period.time_end >= time_start;
                        let time_end_between = period.time_start <= time_end && period.time_end >= time_end;
                        let time_covers = time_start <= period.time_start && time_end >= period.time_end;
                        let conflict = time_start_between || time_end_between || time_covers;
                        if conflict && loc.as_str() == room && period.days.contains(&day) {
                            clash.push(course.clone());
                        }
                    }
                }
            }
        }
        clash
    }
}

impl FindEmptyRooms for models::CourseDB {

    fn find_empty_rooms(&self, time_start: u16, time_end: u16, day: models::Day) -> Vec<String> {
        let mut rooms = HashSet::new();
        for course in &self.courses {
            for section in &course.sections {
                for period in &section.periods {
                    if let Some(loc) = period.location.as_ref() {
                        rooms.insert(loc);
                    }
                }
            }
        }
        let mut valid = rooms.iter().cloned().filter(|room| {
            self.find_course_in_room_at_time(room, time_start, day).len() == 0
        }).collect::<Vec<String>>();
        valid.sort();
        valid
    }

}