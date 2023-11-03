use indicatif::{ProgressBar, ProgressStyle};
use tantivy::Index;

use crate::config::consts::CONFIG_PATH;
use crate::db::entity::Song;
use crate::utils::file::FileUtils;

pub struct TantivyDB {}

impl TantivyDB {}

pub struct MaimaiDB {}

impl MaimaiDB {
    pub(crate) fn update_database(songs: &Vec<Song>) {
        let progress_bar = ProgressBar::new(songs.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:50.green/white} 歌曲数量: {pos}/{len} [{elapsed_precise}]")
                .unwrap(),
        );

        for song in songs {
            // TODO tantivy 更新数据
            progress_bar.inc(1);
        }
        progress_bar.finish();
    }

    /// 删除表并重新创建
    pub fn re_create_table() {}

    /// 初始化数据库
    pub fn init() -> tantivy::Result<()> {
        let index_path = &CONFIG_PATH.join("data");
        FileUtils::create_not_exists(index_path);
        let mut index;
        if index_path.exists() {
            FileUtils::create_not_exists(index_path);
            index = Index::create_in_dir(index_path, Song::get_schema())?;
        } else {
            index = Index::open_in_dir(index_path)?
        }
        Ok(())
    }

    /// 按照传入的 ID 查询歌曲,预期返回值为 1
    pub fn search_song(id: usize) -> Option<Song> {
        None
    }

    /// 按照传入的 SQL 查询歌曲列表
    pub fn search_songs(keyword: String) -> Vec<Song> {
        Vec::new()
    }
}
