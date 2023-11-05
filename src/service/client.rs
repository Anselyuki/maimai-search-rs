use std::process::exit;

use log::{error, info};

use crate::config::consts::PROFILE;
use crate::db::database::MaimaiDB;
use crate::db::entity::Song;

pub struct DXProberClient {}

/// 用于查询歌曲
impl DXProberClient {
    pub(crate) fn get_song_metadata() -> Vec<Song> {
        let url = &PROFILE.remote_api.json_url;
        info!("正在从[{}]下载谱面信息", url);
        let result = match reqwest::blocking::get(url) {
            Ok(response) => response.json::<Vec<Song>>(),
            Err(error) => {
                error!("获取服务器信息出错:{:?}", error);
                exit(exitcode::UNAVAILABLE)
            }
        };
        match result {
            Ok(songs) => songs,
            Err(error) => {
                error!("解析服务器信息出错:{:?}", error);
                exit(exitcode::IOERR)
            }
        }
    }

    /// 按照 id 查询歌曲
    pub fn search_songs_by_id(id: usize) -> Option<Song> {
        MaimaiDB::search_song_by_id(id)
    }

    /// 按照名称查询歌曲
    pub fn search_songs_by_name(name: &str, count: usize) -> Vec<Song> {
        MaimaiDB::search_songs_by_title(name, count)
    }
}
