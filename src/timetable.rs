pub mod repository;
pub mod scheduler;
pub mod api;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;

#[derive(Serialize, Deserialize, Clone)]
pub struct Timetable {
    pub descriptor: TimetableDescriptor,
    pub activities: Vec<Activity>,
    #[serde(with = "ts_seconds")]
    pub update_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimetableDescriptor {
    pub id: TimetableId,
    pub name: String,
    pub variant: TimetableVariant,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct TimetableId {
    pub namespace: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TimetableVariant {
    Semester(u32),
    Year(u32),
    Unique,
}

impl Timetable {
    pub fn new(descriptor: TimetableDescriptor, activities: Vec<Activity>, update_time: DateTime<Utc>) -> Timetable {
        Timetable {
            descriptor,
            activities,
            update_time,
        }
    }
}

impl TimetableDescriptor {
    pub fn new(id: TimetableId, name: String, variant: TimetableVariant) -> TimetableDescriptor {
        TimetableDescriptor {
            id,
            name,
            variant,
        }
    }
}

impl TimetableId {
    pub fn new(namespace: String, id: String) -> TimetableId {
        TimetableId { namespace, id }
    }
}

impl Display for TimetableId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub teacher: Option<String>,
    pub occurrence: ActivityOccurrence,
    pub group: ActivityGroup,
    pub time: ActivityTime,
    pub room: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ActivityOccurrence {
    Regular {
        weekday: Weekday,
    },
    Special {
        date: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActivityGroup {
    pub symbol: String,
    pub name: String,
    pub id: u8,
    pub number: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActivityTime {
    pub start_time: String,
    pub end_time: String,
    pub duration: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<u8> for Weekday {
    fn from(number: u8) -> Self {
        match number {
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => Weekday::Sunday
        }
    }
}


impl From<Weekday> for u8 {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
            Weekday::Sunday => 7
        }
    }
}