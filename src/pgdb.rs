
use postgres::{Config, NoTls, Error};
use crate::models::{CourseDB, Period, Day};
use std::collections::{HashSet, HashMap};
use std::hash::{Hash, Hasher};
use inflector::cases::titlecase;
use bimap::BiHashMap as BiMap;
use std::ops::Deref;

pub fn push_db(db: CourseDB, con: Config) -> Result<(), String> {
    if let Err(err) = push_db_res(db, con) {
        Err(format!("{:?}", err))
    } else {
        Ok(())
    }
}

fn opt_loc_to_building_num(opt: &Option<String>) -> (String, String) {
    let loc = opt.as_ref().unwrap();
    let spl = loc.rfind(" ").unwrap();
    let spl: (&str, &str) = loc.split_at(spl);
    let building = titlecase::to_title_case(spl.0);
    let num = &spl.1[1..];
    (building, num.to_owned())
}

fn push_db_res(db: CourseDB, con: Config) -> Result<(), Error> {
    let mut client = con.connect(NoTls)?;
    let depts = get_depts(&db);
    let buildings = get_buildings(&db);
    let mut trans = client.transaction()?;
    {
        for dept in &depts {
            trans.execute("INSERT INTO departments (id, short_name) VALUES ($1, $2);", &[&(*dept.0 as i32), &dept.1])?;
        }
    }

    {
        for building in &buildings {
            trans.execute("INSERT INTO buildings (id, name) VALUES ($1, $2);", &[&(*building.0 as i32), &building.1])?;
        }
    }

    let mut rooms = BiMap::<i32, (i32, String)>::new();
    let mut days = HashMap::new();

    {
        let mut course_id = 1i32;
        let mut sect_id = 1i32;
        let mut per_id = 1i32;
        let mut room_id = 1;
        let mut days_id = 1;
        for course in &db.courses {
            trans.execute("INSERT INTO courses (id, name, dept, num) VALUES ($1, $2, $3, $4);",
                &[&course_id, &titlecase::to_title_case(course.name.as_str()),
                    &(*depts.get_by_right(&course.dept).unwrap() as i32), &(course.num as i32)]
            )?;
            for sect in &course.sections {
                trans.execute("INSERT INTO sections (id, crn, num, course) VALUES ($1, $2, $3, $4);", &[
                    &sect_id, &(sect.crn as i32), &(sect.num as i16), &course_id
                ])?;

                for per in &sect.periods {
                    if !(per.location.is_some() && per.location.as_ref().unwrap() != "TBA") {continue;}
                    let (building, room) = opt_loc_to_building_num(&per.location);
                    let building_id = *buildings.get_by_right(&building).unwrap() as i32;
                    let key = (building_id, room);
                    let existing_room: Option<&i32> = rooms.get_by_right(&key);
                    let room_id = if existing_room.is_none() {
                        trans.execute("INSERT INTO rooms (id, building_id, room) VALUES ($1, $2, $3);", &[
                            &room_id, &key.0, &key.1
                        ])?;
                        rooms.insert(room_id, key);
                        room_id += 1;
                        room_id - 1
                    } else {
                        *existing_room.unwrap()
                    };

                    let day_key: DaysDb = (&per.days).into();
                    let existing_days = days.get(&day_key);
                    let day_id = if existing_days.is_none() {

                        trans.execute("INSERT INTO days (id, monday, tuesday, wednesday, thursday, friday) VALUES ($1, $2, $3, $4, $5, $6);", &[
                            &days_id, &day_key.monday, &day_key.tuesday, &day_key.wednesday, &day_key.thursday, &day_key.friday
                        ])?;
                        days.insert(day_key, days_id);
                        days_id += 1;
                        days_id - 1
                    } else {
                        *existing_days.unwrap()
                    };
                    trans.execute("INSERT INTO periods (id, section, time_start, time_end, location, days, type) VALUES ($1, $2, $3, $4, $5, $6, $7);", &[
                        &per_id,
                        &sect_id,
                        &(per.time_start as i32),
                        &(per.time_end as i32),
                        &room_id,
                        &day_id,
                        &Option::<i32>::None
                    ])?;

                    per_id += 1;
                }

                sect_id += 1;
            }

            course_id += 1;
        }


        // let periods = get_rooms(&db)
        //     .into_iter()
        //     .enumerate()
        //     .map(|a| ((a.0 + 1) as i32, a.1))
        //     .collect::<BiMap<i32, RefPeriod>>();
        // for (id, per) in periods.iter() {
        //     let (building, number) = {
        //         let loc = per.location.as_ref().unwrap();
        //         let spl = loc.rfind(" ").unwrap();
        //         let spl: (&str, &str) = loc.split_at(spl);
        //         let building = titlecase::to_title_case(spl.0);
        //         let num = &spl.1[1..];
        //         (building, num.to_owned())
        //     };
        //     trans.execute("INSERT INTO rooms (id, building_id, room) VALUES ($1, $2, $3);", &[
        //         id, &(*buildings.get_by_right(&building).unwrap() as i32), &number
        //     ])?;
        //     trans.execute("INSERT INTO days (id, monday, tuesday, wednesday, thursday, friday) VALUES ($1, $2, $3, $4, $5, $6);",
        //     &[
        //         id,
        //         &per.days.contains(&Day::Monday),
        //         &per.days.contains(&Day::Tuesday),
        //         &per.days.contains(&Day::Wednesday),
        //         &per.days.contains(&Day::Thursday),
        //         &per.days.contains(&Day::Friday),
        //     ]
        //     )?;
        // }
    }

    trans.commit()?;


    Ok(())
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct DaysDb {
    monday: bool,
    tuesday: bool,
    wednesday: bool,
    thursday: bool,
    friday: bool,
}

impl From<&Vec<Day>> for DaysDb {
    fn from(days: &Vec<Day>) -> Self {
        let monday = days.contains(&Day::Monday);
        let tuesday = days.contains(&Day::Tuesday);
        let wednesday = days.contains(&Day::Wednesday);
        let thursday = days.contains(&Day::Thursday);
        let friday = days.contains(&Day::Friday);
        DaysDb{monday, tuesday, wednesday, thursday, friday}
    }
}


struct RefPeriod<'a> {
    period: &'a Period,
    pub id: i32,
}

impl<'a> Deref for RefPeriod<'a> {
    type Target = &'a Period;

    fn deref(&self) -> &Self::Target {
        &self.period
    }
}
impl<'a> PartialEq for RefPeriod<'a> {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}
impl<'a> Eq for RefPeriod<'a> {}

impl<'a> Hash for RefPeriod<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

fn get_buildings(db: &CourseDB) -> BiMap<usize, String> {
    to_bimap(db.courses.iter()
        .flat_map(|a| a.sections
            .iter()
            .flat_map(|a| a.periods
                .iter().filter_map(|a| a.location.clone())))
        .filter(|a| a != "TBA")
        .filter_map(|a| {
            if let Some(spl) = a.rfind(" ") {
                let left = a.split_at(spl).0.to_owned();
                Some(left)
            } else {
                None
            }
        })
        .map(|loc| titlecase::to_title_case(loc.as_str()))
        .collect())
}

fn get_depts(db: &CourseDB) -> BiMap<usize, String> {
    to_bimap(db.courses.iter().map(|a| a.dept.clone())
        .collect())
}

fn to_bimap(set: HashSet<String>) -> BiMap<usize, String> {
    let mut i = 1usize;
    let mut bi = BiMap::new();
    for item in set {
        bi.insert(i, item);
        i += 1;
    }
    bi
}

#[cfg(test)]
mod tests {
    use crate::pgdb::{get_buildings, get_depts, push_db_res};
    use postgres::Config;
    use std::str::FromStr;

    #[test]
    fn test_get_buildings() {
        let db = crate::load_db("course-dbs/202001.json");
        let v = get_buildings(&db);
        v.iter().for_each(|e| println!("{:?}", e));
    }

    #[test]
    fn test_get_depts() {
        let db = crate::load_db("course-dbs/202001.json");
        let v = get_depts(&db);
        v.iter().for_each(|e| println!("{:?}", e));
    }

    #[test]
    fn execute() {
        let db = crate::load_db("course-dbs/202001.json");
        let con = Config::from_str("host=192.168.1.75 user=postgres password=72622544 dbname=rpi_classes").unwrap();
        push_db_res(db, con).unwrap();
    }
}
