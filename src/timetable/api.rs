use std::sync::{Arc, Mutex};

use warp::{Filter, path};
use warp::http::StatusCode;

use crate::timetable::repository::{TimetableId, TimetableRepository};

pub fn timetable_api<F>(repository: Arc<Mutex<TimetableRepository>>)
{

    path!("timetable" / String)
        .map(|id: String| {
            let repository = repository.lock().unwrap();
            let timetable = repository.get(TimetableId(id));
            match timetable {
                Some(value) => warp::reply::with_status(serde_json::to_string(value).unwrap(), StatusCode::OK),
                _ => warp::reply::with_status("".to_string(), StatusCode::NOT_FOUND),
            }
        });
}