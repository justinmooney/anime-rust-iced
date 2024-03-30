use std::error::Error;

use rusqlite::{Connection, Result};

const DBPATH: &str = "/home/justin/gotest/anime.db";

#[derive(Clone, Debug)]
pub struct Anime {
    pub title: String,
    pub synopsis: String,
    pub start_date: String,
    pub end_date: String,
}

impl Anime {
    pub fn new(title: String, synopsis: String, start_date: String, end_date: String) -> Anime {
        Anime {
            title: title.replace('\"', ""),
            synopsis,
            start_date,
            end_date,
        }
    }
}

impl Default for Anime {
    fn default() -> Self {
        Self::new(
            String::from("empty_title"),
            String::from("empty_synopsis"),
            String::from("empty_start_date"),
            String::from("empty_end_date"),
        )
    }
}

pub fn load_data() -> Result<Vec<Anime>, Box<dyn Error>> {
    let conn = Connection::open(DBPATH)?;

    let mut stmt = conn.prepare(
        "SELECT Title, Synopsis, StartDate, EndDate
        FROM Animes
        ORDER BY Title
        LIMIT 1000
    ",
    )?;

    let anime_iter = stmt.query_map([], |row| {
        Ok(Anime::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    })?;

    let mut animes: Vec<Anime> = Vec::new();
    for anime in anime_iter {
        animes.push(anime.unwrap());
    }

    Ok(animes)
}
