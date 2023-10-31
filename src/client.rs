use std::collections::{HashMap, HashSet};
use std::process::exit;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use jieba_rs::Jieba;
use levenshtein::levenshtein;
use serde::{Deserialize, Serialize};

use crate::database::MaimaiDB;

pub struct DXProberClient {}

/// 歌曲
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    /// 歌曲 ID
    pub id: String,
    /// 歌曲标题
    pub title: String,
    /// 歌曲类型
    #[serde(rename = "type")]
    pub song_type: String,
    /// 谱面定数
    pub ds: Vec<f32>,
    /// 谱面等级
    pub level: Vec<String>,
    /// 谱面 ID
    pub cids: Vec<u32>,
    /// 谱面详情
    pub charts: Vec<Chart>,
    /// 基本信息
    pub basic_info: BasicInfo,
}

/// 谱面
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chart {
    /// Note 数量分布
    pub notes: Vec<u32>,
    /// 谱面作者
    pub charter: String,
}

/// 基本信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BasicInfo {
    /// 歌曲标题
    pub title: String,
    /// 歌曲作者
    pub artist: String,
    /// 分区类型
    pub genre: String,
    /// 歌曲 BPM
    pub bpm: u32,
    /// 发布时间
    pub release_date: String,
    /// 收录版本
    pub from: String,
    /// 是否为新歌
    pub is_new: bool,
}

// 用于从服务器更新谱面信息
impl DXProberClient {
    /// 更新谱面信息，删除表重新建比较快
    pub fn update_data(url: &String) {
        println!("{}: 正在从[{}]下载谱面信息", "info".green().bold(), url);
        MaimaiDB::re_create_table();

        let songs = match reqwest::blocking::get(url) {
            Ok(response) => { response.json::<Vec<Song>>() }
            Err(error) => panic!("获取服务器信息出错:{:?}", error)
        }.unwrap();

        let progress_bar = ProgressBar::new(songs.len() as u64);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:75.green/white} {pos}/{len} [{elapsed_precise}]").unwrap()
        );
        let connection = MaimaiDB::get_connection();
        let mut statement = connection.prepare_cached("INSERT INTO songs (id, title, song_type, ds, level, cids, charts, basic_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)").expect("SQL 解析失败");
        for song in &songs {
            statement.execute(&[
                &song.id,
                &song.title,
                &song.song_type,
                &serde_json::to_string(&song.ds).unwrap(),
                &serde_json::to_string(&song.level).unwrap(),
                &serde_json::to_string(&song.cids).unwrap(),
                &serde_json::to_string(&song.charts).unwrap(),
                &serde_json::to_string(&song.basic_info).unwrap()
            ]).unwrap();
            progress_bar.inc(1);
        }
        progress_bar.finish();
    }

    /// 按照 id 查询歌曲
    pub fn search_songs_by_id(id: usize) -> Vec<Song> {
        let sql = format!("SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where id = {};", id);
        match MaimaiDB::search_song(sql) {
            None => { Vec::new() }
            Some(song) => { vec![song] }
        }
    }

    /// 按照名称查询歌曲
    pub fn search_songs_by_name(name: &str, count: usize) -> Vec<Song> {
        let stop_words: HashSet<String> = ["的", " ", "!", "\"", "“", "”", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "=", "+", "[", "]", "{", "}", ";", ":", "<", ">", ",", ".", "/", "?"].iter().map(|&s| s.to_string()).collect();
        let keywords: Vec<String> = Jieba::new().cut(name, true).iter()
            .map(|s| String::from(*s))
            // 删除停用词
            .filter(|w| !stop_words.contains(w))
            .collect();

        let mut partial_song = HashMap::new();
        for keyword in keywords {
            let sql = format!("SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where title like '%{}%';", keyword);
            for song in MaimaiDB::search_song_list(sql.as_str()) {
                let id = song.clone().id;
                partial_song.insert(id, song);
            }
        }
        let songs = Self::similar_list_top(partial_song, name, count);
        if songs.is_empty() {
            println!("{}: 查询关键字[{}]找不到匹配项", "warning".red().bold(), name);
            exit(exitcode::OK);
        }
        songs
    }

    /// 模糊查询前 count 的匹配值
    fn similar_list_top(partial_song: HashMap<String, Song>, name: &str, count: usize) -> Vec<Song> {
        // 计算 Levenshtein 距离，并排序
        let mut songs: Vec<(usize, Song)> = partial_song.iter()
            .map(|(_, song)| { (levenshtein(name, &*song.title), song.clone()) })
            .filter(|tuple| (tuple.0 < 100)).collect();
        songs.sort_by(|a, b| a.0.cmp(&b.0));
        // 选择前5个匹配项
        songs.into_iter()
            .take(count)
            .map(|(_, song)| song)
            .collect()
    }
}