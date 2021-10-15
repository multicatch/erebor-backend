pub struct Timetable {
    pub name: String,
    pub activities: Vec<Activity>,
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