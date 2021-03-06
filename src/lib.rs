use error::MensaError;
use mensa::Plan;

use reqwest::{StatusCode, Url};

pub mod error;
pub mod mensa;

pub const API_URL: &'static str = "https://www.swfr.de/index.php";
pub const DEFAULT_QUERY: &'static str = "id=1400&type=98";

pub async fn request_rempart(key: &str) -> Result<Plan, MensaError> {
    request_mensa(UrlBuilder::new(key).set_place(&MensaPlace::Rempartstraße)).await
}

pub async fn request_mensa(url: &mut UrlBuilder) -> Result<Plan, MensaError> {
    let url = url.build();
    let res = match reqwest::get(&url).await {
        Ok(res) => res,
        Err(why) => {
            return Err(MensaError::from(format!(
                "failed to send api request: {:?}",
                why
            )));
        }
    };

    let status = res.status();

    let body = match res.text().await {
        Ok(body) => body,
        Err(why) => {
            return Err(MensaError::from(format!("failed to get body: {:?}", why)));
        }
    };
    if !status.eq(&StatusCode::OK) {
        return Err(MensaError::from(format!(
            "status code not OK: {:?}",
            status
        )));
    }

    let plan: Plan = quick_xml::de::from_str(&body)?;

    Ok(plan)
}

pub struct UrlBuilder {
    api_key: String,
    url: Url,
    query: String,
}

impl UrlBuilder {
    pub fn new(api_key: &str) -> UrlBuilder {
        UrlBuilder {
            api_key: String::from(api_key),
            url: Url::parse(API_URL).unwrap(),
            query: String::from(DEFAULT_QUERY),
        }
    }

    pub fn new_empty_query(api_key: &str) -> UrlBuilder {
        UrlBuilder {
            api_key: String::from(api_key),
            url: Url::parse(API_URL).unwrap(),
            query: String::new(),
        }
    }

    pub fn set_place<'a>(&'a mut self, place: &MensaPlace) -> &'a mut UrlBuilder {
        self.add_query_para("tx_swfrspeiseplan_pi1[ort]", &id(place));
        self
    }

    pub fn add_query_para<'a>(&'a mut self, name: &str, value: &str) -> &'a mut UrlBuilder {
        if self.query.is_empty() {
            self.query = format!("{}={}", name, value);
        } else {
            self.query = format!("{}&{}={}", self.query, name, value);
        }
        self
    }

    pub fn build(&mut self) -> String {
        self.add_query_para(
            "tx_swfrspeiseplan_pi1[apiKey]",
            String::from(&self.api_key).as_str(),
        );
        self.url.set_query(Some(&self.query));
        self.url.to_string()
    }
}

pub enum MensaPlace {
    Rempartstraße,
    Institutsviertel,
    Littenweiler,
    Flugplatz,
}

pub fn id(mensa: &MensaPlace) -> String {
    match mensa {
        MensaPlace::Rempartstraße => String::from("610"),
        MensaPlace::Institutsviertel => String::from("620"),
        MensaPlace::Littenweiler => String::from("630"),
        MensaPlace::Flugplatz => String::from("681"),
    }
}
