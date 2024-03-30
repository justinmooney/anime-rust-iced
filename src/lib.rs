use duckdb::{params, Connection};
// use error_chain::error_chain;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::{thread, time::Duration};

const DBFILE: &str = "./anime.db";

#[derive(Clone, Debug)]
pub struct AnimeItem {
    pub title: String,
    pub synopsis: String,
    pub start_date: String,
    pub end_date: String,
}

impl AnimeItem {
    pub fn new(title: String, synopsis: String, start_date: String, end_date: String) -> AnimeItem {
        AnimeItem {
            title: title.replace('\"', ""),
            synopsis,
            start_date,
            end_date,
        }
    }
}

impl Default for AnimeItem {
    fn default() -> Self {
        Self::new(
            String::from("empty_title"),
            String::from("empty_synopsis"),
            String::from("empty_start_date"),
            String::from("empty_end_date"),
        )
    }
}

fn init_database() -> Result<(), duckdb::Error> {
    let conn = Connection::open(DBFILE)?;
    conn.execute_batch(
        "CREATE TABLE animes (
            title VARCHAR,
            synopsis VARCHAR,
            start_date VARCHAR,
            end_date VARCHAR
        )",
    )?;

    Ok(())
}

pub fn load_data() -> Result<Vec<AnimeItem>, Box<dyn Error>> {
    if !Path::new(DBFILE).exists() {
        init_database()?;
        fetch_animes()?;
    }

    let conn = Connection::open(DBFILE)?;

    let mut stmt = conn.prepare(
        "SELECT title, synopsis, start_date, end_date
        FROM animes
        ORDER BY title
        LIMIT 1000
    ",
    )?;

    let anime_iter = stmt.query_map([], |row| {
        Ok(AnimeItem::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    })?;

    let mut animes: Vec<AnimeItem> = Vec::new();
    for anime in anime_iter {
        animes.push(anime.unwrap());
    }

    Ok(animes)
}

// error_chain! {
//     foreign_links {
//         Io(std::io::Error);
//         HttpRequest(reqwest::Error);
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeListResponse {
    data: Vec<AnimeData>,
    links: Links,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    next: String,
    last: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeData {
    id: String,
    attributes: Attributes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    slug: String,
    synopsis: Option<String>,
    titles: Titles,
    canonical_title: String,
    average_rating: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    episode_count: Option<i64>,
    status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Titles {
    en: Option<String>,
    en_jp: Option<String>,
    ja_jp: Option<String>,
}

fn do_request(url: String) -> Result<AnimeListResponse, Box<dyn Error>> {
    let mut res = reqwest::blocking::get(url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let animes = serde_json::from_str(&body).unwrap();
    Ok(animes)
}

fn insert_anime_batch(anime_list: AnimeListResponse) -> Result<(), duckdb::Error> {
    let conn = Connection::open(DBFILE)?;
    for anime in anime_list.data.into_iter() {
        conn.execute(
            "INSERT INTO animes (title, synopsis, start_date, end_date) VALUES (?,?,?,?)",
            params![
                anime.attributes.canonical_title,
                anime.attributes.synopsis.unwrap_or("".to_owned()),
                anime.attributes.start_date.unwrap_or("".to_owned()),
                anime.attributes.end_date.unwrap_or("".to_owned()),
            ],
        )?;
    }
    Ok(())
}

fn fetch_animes() -> Result<(), Box<dyn Error>> {
    let mut url = String::from("https://kitsu.io/api/edge/anime");
    let mut i = 0;

    loop {
        println!("Requesting: {url}");
        let batch = do_request(url)?;

        url = batch.links.next.clone();
        let last = batch.links.last.clone();

        insert_anime_batch(batch)?;
        if url == last {
            break;
        }

        thread::sleep(Duration::from_millis(50));
        i += 1;
        if i > 10 {
            break;
        }
    }
    println!("Requesting: {url}");
    let batch = do_request(url)?;
    insert_anime_batch(batch)?;

    Ok(())
}
