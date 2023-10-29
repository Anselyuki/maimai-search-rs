use std::collections::HashMap;

use indicatif::{ProgressBar, ProgressStyle};
use jieba_rs::Jieba;
use levenshtein::levenshtein;

use crate::database::MaimaiDB;
use crate::entity::Song;

pub struct DXProberClient {}

// 用于从服务器更新谱面信息
impl DXProberClient {
    /// 更新谱面信息，删除表重新建比较快
    pub fn update_data(url: String) {
        println!("正在从[{}]下载谱面信息", url);
        MaimaiDB::re_create_table();

        let songs = match reqwest::blocking::get(url) {
            Ok(response) => { response.json::<Vec<Song>>() }
            Err(error) => panic!("获取服务器信息出错:{:?}", error)
        }.expect("Json 解析出错");

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
        progress_bar.finish_with_message("更新成功");
    }

    /// 按照 id 查询歌曲
    pub fn search_songs_by_id(id: &str) -> Option<Song> {
        let sql = "SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where id = ?;";
        MaimaiDB::search_song(id, sql)
    }

    /// 按照名称查询歌曲
    pub fn search_songs_by_name(name: &str) -> Vec<Song> {
        let songs = search_songs_by_name_fuzzy(cut(name), name);
        if songs.is_empty() {
            println!("查询的歌曲[{}]找不到匹配项", name)
        }
        songs
    }
}

/// 分词
fn cut(song_name: &str) -> Vec<String> {
    let keywords = Jieba::new().cut(song_name, true);
    keywords.iter()
        .map(|s| String::from(*s))
        // 过滤掉仅包含空格的字符串
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// 模糊查询歌曲
pub fn search_songs_by_name_fuzzy(keywords: Vec<String>, name: &str) -> Vec<Song> {
    let mut partial_song = HashMap::new();
    for keyword in keywords {
        let sql = format!("SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where title like '%{}%';", keyword);
        let songs = MaimaiDB::search_song_list(sql.as_str());
        for song in songs {
            partial_song.insert(song.clone().title, song);
        }
    }
    // 模糊查询前 5 的匹配值
    similar_list_top5(partial_song, name)
}

/// 模糊查询前 5 的匹配值
fn similar_list_top5(partial_song: HashMap<String, Song>, name: &str) -> Vec<Song> {
    // 计算 Levenshtein 距离，并排序
    let mut songs: Vec<(usize, Song)> = partial_song.iter()
        .map(|(title, song)| {
            let distance = levenshtein(name, title);
            (distance, song.clone())
        }).collect();

    songs.sort_by(|a, b| a.0.cmp(&b.0));
    // 选择前5个匹配项
    songs.into_iter()
        .take(3)
        .map(|(_, song)| song)
        .collect()
}
