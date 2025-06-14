use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Read;
use std::path::Path;

const DBFILE: &str = "./anime.db";
const BASEURL: &str = "https://kitsu.io/api/edge/anime";

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn exists() -> bool {
        Path::new(DBFILE).exists()
    }

    pub fn connect() -> Result<Database, duckdb::Error> {
        let conn = Connection::open(DBFILE)?;
        Ok(Database { conn })
    }

    pub fn init(&self) -> duckdb::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE animes (
                title VARCHAR,
                synopsis VARCHAR,
                start_date VARCHAR,
                end_date VARCHAR,
                cover_image VARCHAR
            )",
        )
    }

    fn insert_anime_batch(&self, anime_list: AnimeListResponse) {
        for anime in anime_list.data.into_iter() {
            let img = match anime.attributes.cover_image {
                Some(covers) => covers.original.unwrap_or("".to_owned()),
                None => "".to_owned(),
            };
            self.conn
                .execute(
                    "INSERT INTO animes (title, synopsis, start_date, end_date, cover_image) VALUES (?,?,?,?,?)",
                    params![
                        anime.attributes.canonical_title,
                        anime.attributes.synopsis.unwrap_or("".to_owned()),
                        anime.attributes.start_date.unwrap_or("".to_owned()),
                        anime.attributes.end_date.unwrap_or("".to_owned()),
                        img,
                    ],
                )
                .unwrap();
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimeItem {
    pub title: String,
    pub synopsis: String,
    pub start_date: String,
    pub end_date: String,
    pub cover_image: String,
}

impl AnimeItem {
    pub fn new(
        title: String,
        synopsis: String,
        start_date: String,
        end_date: String,
        cover_image: String,
    ) -> AnimeItem {
        AnimeItem {
            title: title.replace('\"', ""),
            synopsis,
            start_date,
            end_date,
            cover_image,
        }
    }

    pub fn display_date(&self) -> String {
        let start_date = self.start_date.replace("-", "/");
        let end_date = self.end_date.replace("-", "/");

        if start_date == end_date {
            format!("({})", start_date)
        } else if !start_date.is_empty() & end_date.is_empty() {
            format!("({} - ongoing)", start_date)
        } else {
            format!("({} - {})", start_date, end_date)
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
            String::from("empty_cover_image"),
        )
    }
}

#[derive(Debug)]
pub struct AnimeItemList {
    pub items: Vec<AnimeItem>,
}

impl AnimeItemList {
    pub fn new() -> AnimeItemList {
        AnimeItemList { items: Vec::new() }
    }

    pub fn add(&mut self, a: AnimeItem) {
        self.items.push(a);
    }

    pub fn length(&self) -> usize {
        self.items.len()
    }
}

impl Default for AnimeItemList {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_data() -> Result<AnimeItemList, Box<dyn Error>> {
    let conn = Connection::open(DBFILE)?;

    let mut stmt = conn.prepare(
        "SELECT title, synopsis, start_date, end_date, cover_image
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
            row.get(4)?,
        ))
    })?;

    let mut animes = AnimeItemList::new();
    for anime in anime_iter {
        animes.add(anime.unwrap());
    }

    Ok(animes)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeListResponse {
    data: Vec<AnimeData>,
    links: Links,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    first: String,
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
    cover_image: Option<CoverImages>,
    canonical_title: String,
    average_rating: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    episode_count: Option<i64>,
    status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoverImages {
    original: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Titles {
    en: Option<String>,
    en_jp: Option<String>,
    ja_jp: Option<String>,
}

fn do_request(url: &String) -> AnimeListResponse {
    let mut res = reqwest::blocking::get(url).unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    serde_json::from_str(&body).unwrap()
}

#[derive(Debug)]
pub struct AnimeDownloader {
    current: u16,
    database: Database,
    max_pages: u16,
    next_url: String,
    page_size: u16,
    total: u16,
}

pub fn get_downloader() -> Result<AnimeDownloader, Box<dyn Error>> {
    let ad = AnimeDownloader::new()?;
    Ok(ad)
}

impl AnimeDownloader {
    fn new() -> Result<AnimeDownloader, Box<dyn Error>> {
        let db = Database::connect()?;
        db.init()?;

        let url = String::from(BASEURL);
        let resp = do_request(&url);

        let first_url = resp.links.first;
        let last_url = resp.links.last;
        let page_size = 10;

        // TODO lol
        // url like: .../anime?page[offset]=10&page[limit]=10
        let parts: Vec<&str> = last_url.split('&').collect();
        let parts: Vec<&str> = parts[0].split('=').collect();
        let last_offset = parts[1].parse::<u16>().unwrap();
        let total_requests = last_offset / page_size;

        Ok(AnimeDownloader {
            database: db,
            current: 0,
            max_pages: 10,
            page_size,
            next_url: first_url,
            total: total_requests,
        })
    }

    pub fn has_remaining(&self) -> bool {
        self.current < std::cmp::min(self.total, self.max_pages * self.page_size)
    }

    pub fn fetch_next(&mut self) {
        let resp = do_request(&self.next_url);
        self.next_url = resp.links.next.clone();
        self.database.insert_anime_batch(resp);
        self.current += self.page_size;
    }
}
