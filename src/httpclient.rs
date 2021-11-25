use tokio::time::Duration;
use std::fmt::{Display, Formatter};
use reqwest::RequestBuilder;
use rocket::serde::DeserializeOwned;

#[derive(Debug)]
pub enum HttpClientError {
    RequestError(reqwest::Error),
    DeserializationError(serde_json::Error),
    NoData(String),
}

impl Display for HttpClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpClientError::RequestError(e) => write!(f, "Request error. {}", e),
            HttpClientError::DeserializationError(e) => write!(f, "Deserialization error. {}", e),
            HttpClientError::NoData(url) => write!(f, "No data retrieved from {}.", url),
        }
    }
}

pub struct HttpClient {
    client: reqwest::Client,
    max_tries: u16,
    retry_delay: Duration,
}

impl HttpClient {
    pub fn new(max_tries: u16, retry_delay: Duration) -> HttpClient {
        HttpClient {
            client: reqwest::Client::new(),
            max_tries,
            retry_delay
        }
    }

    pub async fn make_retry_request<T, F>(&self, url: String, request: F) -> Result<T, HttpClientError>
        where F: Fn(&reqwest::Client) -> RequestBuilder + Clone,
              T: DeserializeOwned {

        let mut result = Err(HttpClientError::NoData(url.clone()));

        for i in 0..self.max_tries {
            debug!("Making request to {}, try {} / {}", url, i+1, self.max_tries);

            result = self.make_request(request.clone()).await;

            match result {
                Ok(data) => {
                    return Ok(data);
                }
                Err(e) => {
                    warn!("Error during fetch from {} - {} / {}: {}. Retrying in {:?}",
                        url,
                        i+1,
                        self.max_tries,
                        e,
                        self.retry_delay
                    );
                    result = Err(e);
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }

        result
    }

    pub async fn make_request<T, F>(&self, request: F) -> Result<T, HttpClientError>
        where F: Fn(&reqwest::Client) -> RequestBuilder,
              T: DeserializeOwned {
        let result = request(&self.client)
            .send()
            .await?
            .text()
            .await?;

        serde_json::from_str(&result)
            .map_err(|e| {
                error!("Cannot deserialize response. {}", e);
                HttpClientError::DeserializationError(e)
            })
    }
}

impl From<reqwest::Error> for HttpClientError {
    fn from(e: reqwest::Error) -> Self {
        error!("Request error. {}", e);
        HttpClientError::RequestError(e)
    }
}
