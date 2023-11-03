use crate::config::consts::CONFIG_PATH;
use log::error;
use rusqlite::{params, Connection, Row};
use std::fs;
use std::process::exit;

use crate::db::entity::{BasicInfo, Chart, Song};

pub struct MaimaiDB {}

impl MaimaiDB {
    /// 获取数据库连接
    pub fn get_connection() -> Connection {
        let db_path = CONFIG_PATH.join("maimaidxprober.db");
        let conn = Connection::open(&db_path);
        match conn {
            Ok(conn) => conn,
            Err(error) => {
                error!("数据库连接失败:{}", error);
                exit(exitcode::DATAERR);
            }
        }
    }

    /// 删除表并重新创建
    pub fn re_create_table() {
        MaimaiDB::get_connection()
            .execute("drop table if exists songs;", [])
            .unwrap();
        MaimaiDB::init();
    }

    /// 初始化数据库
    pub fn init() {
        fs::create_dir_all(&*CONFIG_PATH).unwrap();
        let connection = MaimaiDB::get_connection();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS songs
                (
                    id         TEXT PRIMARY KEY,
                    title      TEXT,
                    song_type  TEXT,
                    ds         BLOB,
                    level      BLOB,
                    cids       BLOB,
                    charts     BLOB,
                    basic_info BLOB
                );",
                [],
            )
            .expect("创建数据库表失败");
    }

    /// 按照传入的 SQL 查询歌曲,预期返回值为 1
    pub fn search_song(sql: String) -> Option<Song> {
        let connection = MaimaiDB::get_connection();
        let mut statement = connection.prepare(sql.as_str()).unwrap();
        statement.query_row(params![], parse_row).ok()
    }

    /// 按照传入的 SQL 查询歌曲列表
    pub fn search_song_list(sql: &str) -> Vec<Song> {
        let connection = MaimaiDB::get_connection();
        let mut statement = connection.prepare(sql).unwrap();
        statement
            .query_map(params![], parse_row)
            .unwrap()
            .filter_map(Result::ok)
            .collect()
    }
}

/// 数据库列解析为 Song
fn parse_row(row: &Row) -> Result<Song, rusqlite::Error> {
    let song = Song {
        id: row.get::<usize, String>(0).unwrap(),
        title: row.get::<usize, String>(1).unwrap(),
        song_type: row.get::<usize, String>(2).unwrap(),
        ds: serde_json::from_str(row.get::<usize, String>(3).unwrap().as_str())
            .expect("谱面定数序列化错误"),
        level: serde_json::from_str(row.get::<usize, String>(4).unwrap().as_str())
            .expect("谱面等级序列化错误"),
        cids: serde_json::from_str(row.get::<usize, String>(5).unwrap().as_str())
            .expect("谱面ID序列化错误"),
        charts: serde_json::from_str::<Vec<Chart>>(row.get::<usize, String>(6).unwrap().as_str())
            .expect("Chart 序列化错误"),
        basic_info: serde_json::from_str::<BasicInfo>(
            row.get::<usize, String>(7).unwrap().as_str(),
        )
        .expect("basic_info 序列化错误"),
    };
    Ok(song)
}
