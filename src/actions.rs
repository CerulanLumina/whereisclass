use crate::{models, TimeCode};
use std::collections::HashSet;

/// An extension trait to add finding course in room functionality
pub trait FindCourseInRoomAtTime {
    /// Find a course in a room at a given time instant.
    fn find_course_in_room_at_time(
        &self,
        room: &str,
        time: TimeCode,
        day: models::Day,
    ) -> Vec<models::Course> {
        self.find_course_in_room_at_time_range(room, time, time, day)
    }
    /// Find a course in a room for a given range
    fn find_course_in_room_at_time_range(
        &self,
        room: &str,
        time_start: TimeCode,
        time_end: TimeCode,
        day: models::Day,
    ) -> Vec<models::Course>;
}

/// A trait that allows for finding empty rooms for a given time range
pub trait FindEmptyRooms {
    /// Find empty rooms given a start time, and ending time, and a day
    fn find_empty_rooms(
        &self,
        time_start: TimeCode,
        time_end: TimeCode,
        day: models::Day,
    ) -> Vec<String>;
}

impl FindCourseInRoomAtTime for models::CourseDB {
    fn find_course_in_room_at_time_range(
        &self,
        room: &str,
        time_start: TimeCode,
        time_end: TimeCode,
        day: models::Day,
    ) -> Vec<models::Course> {
        let mut clash = Vec::new();
        // Naive impl b/c lazy (whats dp lol)
        for course in &self.courses {
            for section in &course.sections {
                for period in &section.periods {
                    if period.location.is_some() {
                        let loc = period.location.as_ref().unwrap();
                        let time_start_between =
                            period.time_start <= time_start && period.time_end >= time_start;
                        let time_end_between =
                            period.time_start <= time_end && period.time_end >= time_end;
                        let time_covers =
                            time_start <= period.time_start && time_end >= period.time_end;
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
    fn find_empty_rooms(
        &self,
        time_start: TimeCode,
        time_end: TimeCode,
        day: models::Day,
    ) -> Vec<String> {
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
        let mut valid = rooms
            .iter()
            .filter(|room| {
                self.find_course_in_room_at_time_range(room, time_start, time_end, day)
                    .len()
                    == 0
            })
            .map(|a| a.clone().clone())
            .collect::<Vec<_>>();
        valid.sort();
        valid
    }
}
