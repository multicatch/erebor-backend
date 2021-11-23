use erebor_backend::run_scheduler;
use erebor_backend::timetable::repository::{ShareableTimetableProvider};
use log::LevelFilter;
use erebor_backend::timetable::repository::inmemory::in_memory_repo;
use erebor_backend::timetable::api::{get_all_namespaces, get_all_timetables, get_timetable};
use rocket::{routes, Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;

#[rocket::main]
async fn main() {
    env_logger::builder()
        .filter_module("erebor_backend", LevelFilter::Trace)
        .filter_module("rocket", LevelFilter::Info)
        .filter_module("reqwest", LevelFilter::Debug)
        .init();

    let repository = run_scheduler(in_memory_repo).unwrap();

    let result = rocket::build()
        .manage(ShareableTimetableProvider::new(repository))
        .mount("/", routes![get_all_namespaces, get_all_timetables, get_timetable])
        .attach(Cors::default())
        .launch()
        .await;

    match result {
        Ok(_) => println!("Server finished normally."),
        Err(e) => eprintln!("Server crashed. {}", e),
    }
}

struct Cors {
    allowed_origins: Vec<String>
}

impl Default for Cors {
    fn default() -> Cors {
        Cors {
           allowed_origins: vec![
               "https://erebor.vpcloud.eu/".to_string()
           ]
        }
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", self.allowed_origins.join(", ")));
    }
}