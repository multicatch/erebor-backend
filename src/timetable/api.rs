use crate::timetable::repository::inmemory::InMemoryRepo;
use crate::timetable::repository::{TimetableId, TimetableProvider};
use rocket::State;
use rocket::response::{status, content};
use rocket::http::Status;

#[get("/timetable")]
pub fn get_all_timetables(repo: &State<InMemoryRepo>) -> status::Custom<content::Json<String>> {
    serde_json::to_string(&repo.available())
        .map(|json|
            status::Custom(Status::Ok, content::Json(json))
        )
        .unwrap_or_else(|e| {
            error!("Cannot serialize available timetables: {}", e);
            status::Custom(Status::InternalServerError, content::Json("{}".to_string()))
        })
}

#[get("/timetable/<id>")]
pub fn get_timetable(repo: &State<InMemoryRepo>, id: &str) -> status::Custom<content::Json<String>> {
    let timetable = repo.get(TimetableId(id.to_string()));

    timetable
        .map(|value|
            serde_json::to_string(&value)
                .map(|json| (Status::Ok, json))
                .unwrap_or_else(|e| {
                    error!("Cannot serialize timetable [{}]: {}", id, e);
                    (Status::InternalServerError, "{}".to_string())
                })
        )
        .map(|(status, json)|
            status::Custom(status, content::Json(json))
        )
        .unwrap_or_else(||
            status::Custom(Status::NotFound, content::Json("{}".to_string()))
        )
}