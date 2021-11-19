pub mod repository;
pub mod scheduler;
pub mod api;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Timetable {
    pub name: String,
    pub variant: TimetableVariant,
    pub activities: Vec<Activity>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TimetableVariant {
    Semester(u8),
    Year(u8),
    Unique
}

impl Timetable {
    pub fn new(name: String, variant: TimetableVariant, activities: Vec<Activity>) -> Timetable {
        Timetable {
            name,
            variant,
            activities,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Activity {
    Regular {
        weekday: Weekday,
        name: String,
        teacher: String
    },
    Special {
        date: String,
        name: String,
        teacher: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}