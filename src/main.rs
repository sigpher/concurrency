use log::info;
use regex::Regex;
use sqlx::{SqlitePool, error::Error, prelude::FromRow, sqlite::SqliteConnectOptions};
use std::env;
use std::path::Path;
use std::{str::FromStr, time::Duration};

#[derive(Debug, FromRow)]
struct User {
    // id: i64,
    name: String,
    age: Option<i32>,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().unwrap();
    env_logger::init();

    // let db = env::var("DATABASE_URL").expect("database should be set");
    let base_url = env::var("BASE_URL").expect("base url should be set");

    // info!("connecting database: {}", db);
    // let options = SqliteConnectOptions::from_str(&db)?
    //     .create_if_missing(true)
    //     .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
    //     .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
    //     .busy_timeout(Duration::from_secs(5));
    // let pool = SqlitePool::connect_with(options).await?;
    // info!("database {} connected", db);

    // let index_html = scrape_index(&base_url).await;
    // let links = parse_index_get_all_links(index_html).await;

    // let mut standards: Vec<Standard> = Vec::new();

    // for link in links {
    //     // println!("{link}");
    //     let html = scrape_detail(&link).await.unwrap();
    //     let standard = parse_detail_get_standard(&html);
    //     standards.push(standard);
    // }

    // println!("{:?}", standards);

    // for std in standard {
    //     println!("{}", std.title);
    // }

    // let pool = SqlitePool::connect_with(options).await?;

    // let create_db_sql = fs::read_to_string("sql/create_user.sql").unwrap();
    // sqlx::query(&create_db_sql).execute(&pool).await?;

    // batch_insert_users(&pool).await?;
    let link = "http://down.foodmate.net/standard/sort/3/166972.html";
    let html = scrape_detail(link).await.unwrap();
    let standard = parse_detail_get_standard(&html);
    println!("{:?}", standard);

    Ok(())
}

async fn batch_insert_users(pool: &SqlitePool) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let users = vec![
        User {
            // id: 1,
            name: "choi".into(),
            age: Some(30),
            email: "cwy@tdyh.com.cn".into(),
        },
        User {
            // id: 2,
            name: "lora".into(),
            age: Some(26),
            email: "zdq@tdyh.com.cn".into(),
        },
        User {
            // id: 3,
            name: "troy".into(),
            age: Some(6),
            email: "cly@tdyh.com.cn".into(),
        },
        User {
            // id: 4,
            name: "elan".into(),
            age: Some(2),
            email: "cyl@tdyh.com.cn".into(),
        },
    ];

    for user in users.iter() {
        sqlx::query("INSERT OR IGNORE INTO users (name, age, email) VALUES(?, ?, ?)")
            .bind(&user.name)
            .bind(&user.age)
            .bind(&user.email)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    sqlx::query("VACUUM;").execute(pool).await?;

    Ok(())
}

pub fn parse_detail_get_standard(html: &str) -> Standard {
    let item_id_re = Regex::new(r"<script.*?item_id=(?<item_id>\d{3,}),").unwrap();
    let title_re = Regex::new(r#"(?s)title2.*?<span>(?<title>.*?)<font"#).unwrap();
    // let state_re = Regex::new(r#"(?s)<td bgcolor.*?<img src="(?<state_image>.*?)""#).unwrap();
    let status_re = Regex::new(r#"(?s)标准状态.*?<img src="(?<status_image>.*?)""#).unwrap();
    let published_at_re =
        Regex::new(r#"(?s)发布日期.*?(?<published_at>\d{4}-\d{2}-\d{2})"#).unwrap();
    let effective_at_re =
        Regex::new(r#"(?s)实施日期.*?(?<effective_at>\d{4}-\d{2}-\d{2})"#).unwrap();
    let issued_by_re =
        Regex::new(r##"(?s)颁发部门.*?<td bgcolor="#FFFFFF">(?<issued_by>.*?)</td>"##).unwrap();

    let item_id = item_id_re
        .captures(html)
        .unwrap()
        .name("item_id")
        .unwrap()
        .as_str()
        .parse::<i64>()
        .unwrap();

    let title = title_re
        .captures(html)
        .unwrap()
        .name("title")
        .unwrap()
        .as_str()
        .to_string();

    let status = status_re
        .captures(html)
        .unwrap()
        .name("status_image")
        .unwrap()
        .as_str();

    let filename = Path::new(status).file_stem().unwrap().to_str().unwrap();

    let status = match filename {
        "bfyx" => "部分有效".to_string(),
        "jjfz" => "即将废止".to_string(),
        "jjss" => "即将生效".to_string(),
        "xxyx" => "现行有效".to_string(),
        "yjfz" => "已经废止".to_string(),
        "wz" => "未知".to_string(),
        _ => "".to_string(),
    };

    let published_at = published_at_re
        .captures(html)
        .unwrap()
        .name("published_at")
        .unwrap()
        .as_str()
        .to_string();

    let effective_at = effective_at_re
        .captures(html)
        .unwrap()
        .name("effective_at")
        .unwrap()
        .as_str()
        .to_string();

    let issued_by = issued_by_re
        .captures(html)
        .unwrap()
        .name("issued_by")
        .unwrap()
        .as_str()
        .to_string();

    Standard {
        item_id,
        title,
        status,
        published_at,
        effective_at,
        issued_by,
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Standard {
    pub item_id: i64,
    pub title: String,
    pub status: String,
    pub published_at: String,
    pub effective_at: String,
    pub issued_by: String,
}

async fn scrape_page(url: &str) -> Option<String> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0";
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .referer(true)
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("connect website error");

    let resp = client.get(url).send().await.expect("get no response");
    if resp.status().is_success() {
        let html = resp.text_with_charset("gb2312").await.unwrap();
        return Some(html);
    } else {
        None
    }
}

async fn scrape_index(url: &str) -> Option<String> {
    scrape_page(url).await
}

async fn scrape_detail(url: &str) -> Option<String> {
    scrape_page(url).await
}

async fn parse_index_get_all_links(html: Option<String>) -> Vec<String> {
    let re = Regex::new(r#"(?s)class="bz_listl".*?<A.*?href="(?<link>.*?)""#).unwrap();
    re.captures_iter(&html.unwrap())
        .map(|c| c.name("link").unwrap().as_str().to_string())
        .collect()
}
