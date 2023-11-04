use std::process::exit;

use indicatif::{ProgressBar, ProgressStyle};
use log::error;
use tantivy::{Index, IndexReader, IndexWriter};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

use crate::config::consts::{CONFIG_PATH, SONG_SCHEMA};
use crate::db::entity::{Song, SongField};
use crate::utils::file::FileUtils;

/// 新版本使用 Tantivy 作为数据源
pub struct MaimaiDB {}

impl MaimaiDB {
    /// 获取写入器
    pub fn get_writer() -> IndexWriter {
        let index = Self::get_index();
        match index.writer(15000000) {
            Ok(writer) => writer,
            Err(error) => {
                error!("获取写入器时出现错误: {:?}", error);
                exit(exitcode::IOERR)
            }
        }
    }

    pub fn get_reader() -> IndexReader {
        let index = Self::get_index();
        match index.reader() {
            Ok(reader) => reader,
            Err(error) => {
                error!("获取读取器时出现错误: {:?}", error);
                exit(exitcode::IOERR)
            }
        }
    }

    /// 打开或创建索引
    ///
    /// 解耦合主要是为了方便之后重建索引的步骤
    fn get_index() -> Index {
        let index_path = &CONFIG_PATH.join("data");
        let result = if !index_path.exists() {
            // 如果这个目录不存在 Tantivy 就会报错,所以需要手动创建,文件夹里有没有索引倒是次要的
            FileUtils::create_not_exists(index_path);
            Index::create_in_dir(index_path, SONG_SCHEMA.clone())
        } else {
            Index::open_in_dir(index_path)
        };
        match result {
            Ok(index) => index,
            Err(error) => {
                error!("打开或创建索引时出现错误: {:?}", error);
                exit(exitcode::IOERR)
            }
        }
    }

    /// 更新歌曲数据
    pub fn update_database(songs: &Vec<Song>) {
        let progress_bar = ProgressBar::new(songs.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:50.green/white} 歌曲数量: {pos}/{len} [{elapsed_precise}]")
                .unwrap(),
        );

        let mut writer = Self::get_writer();
        for song in songs {
            let document = match song.document() {
                Ok(document) => document,
                Err(error) => {
                    error!("构建歌曲[{}]的文档时出现错误\n[Cause]:{:?}", song.id, error);
                    continue;
                }
            };
            if let Err(error) = writer.add_document(document) {
                error!("添加歌曲[{}]时出现错误: {:?}", song.id, error);
            }
            progress_bar.inc(1);
        }
        if let Err(error) = writer.commit() {
            error!("提交索引时出现错误: {:?}", error);
        }
        progress_bar.finish();
    }

    /// 删除表并重新创建
    pub fn re_create_table() {}

    /// 按照传入的 ID 查询歌曲,预期返回值为 1
    pub fn search_song(id: usize) -> Option<Song> {
        let reader = Self::get_reader();
        let searcher = reader.searcher();

        let query_parser =
            QueryParser::for_index(&Self::get_index(), vec![Song::field(SongField::Id)]);
        let query = query_parser.parse_query(id.to_string().as_str()).unwrap();

        // ID 是唯一的,所以只需要返回一个结果
        let top_docs = match searcher.search(&query, &TopDocs::with_limit(1)) {
            Ok(top_docs) => top_docs,
            Err(error) => {
                error!("查询歌曲[{}]时出现错误\n[Cause]:{:?}", id, error);
                exit(exitcode::IOERR)
            }
        };

        match top_docs.len() {
            0 => None,
            1 => {
                let retrieved_doc = searcher.doc(top_docs[0].1).unwrap();
                Some(match Song::from_document(retrieved_doc) {
                    Ok(song) => song,
                    Err(error) => {
                        error!("反序列化错误\n[Cause]:{:?}", error);
                        exit(exitcode::IOERR)
                    }
                })
            }
            _ => {
                error!("查询歌曲[{}]时出现错误\n[Cause]:{}", id, "查询结果大于 1");
                exit(exitcode::IOERR)
            }
        }
    }

    /// 按照传入的 SQL 查询歌曲列表
    pub fn search_songs(keyword: String) -> Vec<Song> {
        Vec::new()
    }
}
