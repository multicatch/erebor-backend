use rocket::fairing::{Fairing, Kind, Info};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct Cors {
    allowed_origins: Vec<String>,
    fallback_origin: String,
}

impl Cors {
    pub fn new(allowed_origins: &[String], fallback_origin: String) -> Cors {
        let mut allowed_origins = allowed_origins.to_vec();
        allowed_origins.insert(0, fallback_origin.clone());
        Cors {
            allowed_origins,
            fallback_origin,
        }
    }
}

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let http_referer = req.headers().get_one("Referer")
            .unwrap_or("");

        let allowed_origin = if self.allowed_origins
            .iter()
            .any(|origin| http_referer.contains(origin)) {

            http_referer.to_string()
        } else {
            self.fallback_origin.clone()
        };

        res.set_header(Header::new("Access-Control-Allow-Origin", allowed_origin));
    }
}