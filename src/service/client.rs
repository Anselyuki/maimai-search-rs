use std::collections::{HashMap, HashSet};
use std::process::exit;

use crate::db::database::MaimaiDB;
use jieba_rs::Jieba;
use levenshtein::levenshtein;
use log::warn;

use crate::db::entity::Song;

pub struct DXProberClient {}

/// 用于查询歌曲
impl DXProberClient {
    /// 按照 id 查询歌曲
    pub fn search_songs_by_id(id: usize) -> Vec<Song> {
        let sql = format!("SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where id = {};", id);
        match MaimaiDB::search_song(sql) {
            None => Vec::new(),
            Some(song) => {
                vec![song]
            }
        }
    }

    /// 按照名称查询歌曲
    pub fn search_songs_by_name(name: &str, count: usize) -> Vec<Song> {
        let stop_words: HashSet<String> = [
            "的", " ", "!", "\"", "“", "”", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "=",
            "+", "[", "]", "{", "}", ";", ":", "<", ">", ",", ".", "/", "?",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();
        let keywords: Vec<String> = Jieba::new()
            .cut(name, true)
            .iter()
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
            warn!("查询关键字[{}]找不到匹配项", name);
            exit(exitcode::OK);
        }
        songs
    }

    /// 模糊查询前 count 的匹配值
    fn similar_list_top(
        partial_song: HashMap<String, Song>,
        name: &str,
        count: usize,
    ) -> Vec<Song> {
        // 计算 Levenshtein 距离，并排序
        let mut songs: Vec<(usize, Song)> = partial_song
            .iter()
            .map(|(_, song)| (levenshtein(name, &*song.title), song.clone()))
            .filter(|tuple| (tuple.0 < 100))
            .collect();
        songs.sort_by(|a, b| a.0.cmp(&b.0));
        // 选择前5个匹配项
        songs
            .into_iter()
            .take(count)
            .map(|(_, song)| song)
            .collect()
    }
}
