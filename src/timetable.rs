pub struct Timetable {
    pub name: String,
    pub activities: Vec<Activity>,
}

impl Timetable {
    pub fn new(name: String, activities: Vec<Activity>) -> Timetable {
        Timetable {
            name,
            activities,
        }
    }
}

pub enum Activity {
    Regular {
        weekday: Weekday
    },
    Special {
        date: String
    },
}

pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}