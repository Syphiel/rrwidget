use crate::structs::{Config, Item, RedditPost, RedditPostChildData, RedditToken};
use lru::LruCache;
use std::error::Error;
use std::io::Read;

pub fn get_posts(cache: &mut LruCache<String, Vec<u8>>) -> Result<Vec<Item>, Box<dyn Error>> {
    let conf: Config = confy::load("rrwidget", None)?;
    let token = ureq::post("https://www.reddit.com/api/v1/access_token")
        .set("User-Agent", &conf.agent())
        .set("Authorization", &format!("Basic {}", &conf.creds()))
        .send_form(&[
            ("grant_type", "password"),
            ("username", &conf.reddit_user),
            ("password", &conf.reddit_pass),
        ]);

    let token: RedditToken = token?.into_json()?;

    let posts: RedditPost = ureq::get(&format!(
        "https://oauth.reddit.com/r/{}/new",
        &conf.subreddit
    ))
    .set("Authorization", &format!("Bearer {}", token.access_token))
    .set("User-Agent", &conf.agent())
    .query("limit", "10")
    .call()?
    .into_json()?;

    let data: Vec<RedditPostChildData> =
        posts.data.children.iter().map(|x| x.data.clone()).collect();

    let mut items: Vec<Item> = Vec::with_capacity(10);
    for x in &data {
        let mut image_data = x.preview.as_ref().map(|p| {
            if cache.contains(&p.images[0].source.url) {
                return cache.get(&p.images[0].source.url).unwrap().clone();
            }
            let Ok(image) = ureq::get(&p.images[0].source.url)
                .set("User-Agent", &conf.agent())
                .call()
            else {
                return Vec::new();
            };
            let len: usize = match image.header("Content-Length") {
                Some(l) => match l.parse() {
                    Ok(n) => n,
                    Err(_) => return Vec::new(),
                },
                None => return Vec::new(),
            };
            let mut buf = Vec::with_capacity(len);
            match image.into_reader().take(10_000_000).read_to_end(&mut buf) {
                Ok(_) => {
                    cache.put(p.images[0].source.url.clone(), buf.clone());
                    buf
                }
                Err(_) => Vec::new(),
            }
        });
        if let Some(img) = image_data.as_ref() {
            if img.is_empty() {
                image_data = None;
            }
        }
        items.push(Item {
            id: x.name.clone(),
            title: x.title.clone(),
            url: x.url.clone(),
            created: x.created(),
            image_data,
        });
    }

    ureq::post("https://www.reddit.com/api/v1/revoke_token")
        .set("Authorization", &format!("Basic {}", &conf.creds()))
        .send_form(&[("token", &token.access_token)])?;

    Ok(items)
}
