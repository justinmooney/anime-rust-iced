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
    pub fn new() -> Anime {
        Anime {
            title: String::new(),
            synopsis: String::new(),
            start_date: String::new(),
            end_date: String::new(),
        }
    }
}

impl Default for Anime {
    fn default() -> Self {
        Self::new()
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
        let mut title: String = row.get(0)?;
        title = title.replace('\"', "");
        Ok(Anime {
            title,
            synopsis: row.get(1)?,
            start_date: row.get(2)?,
            end_date: row.get(3)?,
        })
    })?;

    let mut animes: Vec<Anime> = Vec::new();
    for anime in anime_iter {
        animes.push(anime.unwrap());
    }

    Ok(animes)

    // let mut anime_string = String::new();
    //
    // for anime in anime_iter {
    //     // println!("Found {:?}", anime);
    //     let aa = anime.unwrap();
    //     anime_string.push_str(&aa.title);
    //     anime_string.push_str("\n");
    // }
    //
    // Ok(anime_string)
}
