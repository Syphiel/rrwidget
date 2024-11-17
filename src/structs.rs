use base64::prelude::*;
use html_escape::decode_html_entities;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use time_humanize::HumanTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct RedditToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedditPost {
    pub kind: String,
    pub data: RedditPostData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedditPostData {
    pub children: Vec<RedditPostChild>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedditPostChild {
    pub kind: String,
    pub data: RedditPostChildData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedditPostChildData {
    pub name: String,
    pub title: String,
    #[serde(deserialize_with = "html_escape")]
    pub url: String,
    pub preview: Option<RedditPostPreview>,
    pub created_utc: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedditPostPreview {
    pub images: Vec<RedditPostImage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedditPostImage {
    pub source: RedditPostImageSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedditPostImageSource {
    #[serde(deserialize_with = "html_escape")]
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub reddit_user: String,
    pub reddit_pass: String,
    pub subreddit: String,
}

impl Config {
    pub fn creds(&self) -> String {
        BASE64_STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
    }

    pub fn agent(&self) -> String {
        format!("linux:rrwidget/0.1 (by /u/{})", self.reddit_user)
    }
}

impl RedditPostChildData {
    pub fn created(&self) -> String {
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        HumanTime::from_duration_since_timestamp(self.created_utc as u64)
            .to_text_en(time_humanize::Accuracy::Rough, time_humanize::Tense::Past)
    }
}

impl Display for RedditPostChildData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "ID: {}\nTitle: {}\nCreated: {}\nURL: {}\nImage URL: {}\n",
            self.name,
            &self.title,
            &self.created(),
            &self.url,
            match &self.preview {
                Some(p) => &p.images[0].source.url,
                None => "",
            }
        )
    }
}

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub url: String,
    pub created: String,
    pub image_data: Option<Vec<u8>>,
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Title: {}\nCreated: {}\nURL: {}\n",
            self.title, self.created, self.url
        )
    }
}

pub fn html_escape<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(decode_html_entities(&s).to_string())
}
