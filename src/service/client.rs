use crate::db::database::MaimaiDB;
use crate::db::entity::Song;

pub struct DXProberClient {}

/// 用于查询歌曲
impl DXProberClient {
    /// 按照 id 查询歌曲
    pub fn search_songs_by_id(id: usize) -> Option<Song> {
        MaimaiDB::search_song_by_id(id)
    }

    /// 按照名称查询歌曲
    pub fn search_songs_by_name(name: &str, count: usize) -> Vec<Song> {
        MaimaiDB::search_songs_by_title(name, count)
    }
}
