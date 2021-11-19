pub mod repository;
pub mod scheduler;
pub mod api;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone)]
pub struct Timetable {
    pub descriptor: TimetableDescriptor,
    pub activities: Vec<Activity>,
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
    Semester(u8),
    Year(u8),
    Unique,
}

impl Timetable {
    pub fn new(descriptor: TimetableDescriptor, activities: Vec<Activity>) -> Timetable {
        Timetable {
            descriptor,
            activities,
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
pub enum Activity {
    Regular {
        weekday: Weekday,
        name: String,
        teacher: String,
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
    Sunday,
}