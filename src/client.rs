use std::collections::{HashMap, HashSet};

use indicatif::{ProgressBar, ProgressStyle};
use jieba_rs::Jieba;
use levenshtein::levenshtein;

use crate::database::MaimaiDB;
use crate::entity::Song;

pub struct DXProberClient {}

// 用于从服务器更新谱面信息
impl DXProberClient {
    /// 更新谱面信息，删除表重新建比较快
    pub fn update_data(url: &String) {
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
    pub fn search_songs_by_name(name: &str, count: usize) -> Vec<Song> {
        let symbols: Vec<String> = vec!["!", " "].iter().map(|&s| s.to_string()).collect();
        let stop_words: HashSet<String> = ["的", "了", "是", "在", "我", "你", "他"].iter().map(|&s| s.to_string()).collect();
        let keywords: Vec<String> = Jieba::new().cut(name, true).iter()
            .map(|s| String::from(*s))
            // 删除停用词
            .filter(|w| !stop_words.contains(w))
            // 删除符号
            .filter(|s| !symbols.contains(s))
            .collect();
        dbg!(&keywords);

        let mut partial_song = HashMap::new();
        for keyword in keywords {
            let sql = format!("SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where title like '%{}%';", keyword);
            for song in MaimaiDB::search_song_list(sql.as_str()) {
                let id = song.clone().id;
                partial_song.insert(id, song);
            }
        }
        let songs = Self::similar_list_top(partial_song, name, count);
        if songs.is_empty() { println!("查询的歌曲[{}]找不到匹配项", name) }
        songs
    }

    /// 模糊查询前 count 的匹配值
    fn similar_list_top(partial_song: HashMap<String, Song>, name: &str, count: usize) -> Vec<Song> {
        // 计算 Levenshtein 距离，并排序
        let mut tuples: Vec<(usize, Song)> = partial_song.iter()
            .map(|(_, song)| { (levenshtein(name, &*song.title), song.clone()) })
            .filter(|tuple| (tuple.0 < 100)).collect();
        tuples.sort_by(|a, b| a.0.cmp(&b.0));

        for tuple in tuples.iter() {
            let distance = &tuple.0;
            let song = &tuple.1;
            println!("[{}] {} - {}", distance, song.title, name);
        }
        // 选择前5个匹配项
        tuples.into_iter()
            .take(count)
            .map(|(_, song)| song)
            .collect()
    }
}