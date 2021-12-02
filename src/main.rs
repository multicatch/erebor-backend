use erebor_backend::run_scheduler;
use erebor_backend::timetable::repository::{ShareableTimetableProvider};
use log::LevelFilter;
use erebor_backend::timetable::api::{get_all_namespaces, get_all_timetables, get_timetable};
use rocket::routes;
use erebor_backend::timetable::repository::sqlite::create_sqlite;
use rusqlite::Connection;
use erebor_backend::cors::Cors;

#[rocket::main]
async fn main() {
    env_logger::builder()
        .filter_module("erebor_backend", LevelFilter::Trace)
        .filter_module("rocket", LevelFilter::Info)
        .filter_module("reqwest", LevelFilter::Debug)
        .init();


    let connection = Connection::open("erebor.db").unwrap();
    let repository = run_scheduler(move || create_sqlite(connection)).unwrap();

    let result = rocket::build()
        .manage(ShareableTimetableProvider::new(repository))
        .mount("/", routes![get_all_namespaces, get_all_timetables, get_timetable])
        .attach(Cors::new(&[], "https://erebor.vpcloud.eu".to_string()))
        .launch()
        .await;

    match result {
        Ok(_) => println!("Server finished normally."),
        Err(e) => eprintln!("Server crashed. {}", e),
    }
}