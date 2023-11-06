use std::fs;
use std::process::exit;

use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::tokenizer::{TokenStream, Tokenizer};
use tantivy::{DocAddress, Index, IndexWriter, Score, Searcher};
use zhconv::{zhconv, Variant};

use crate::config::consts::{CONFIG_PATH, SONG_SCHEMA};
use crate::db::entity::{Song, SongField};
use crate::utils::file::FileUtils;

/// 新版本使用 Tantivy 作为数据源
pub struct MaimaiDB {}

impl MaimaiDB {
    /// 获取写入器
    pub fn get_writer() -> IndexWriter {
        let index = Self::get_index();
        match index.writer(15_000_000) {
            Ok(writer) => writer,
            Err(error) => {
                error!("获取写入器时出现错误: {:?}", error);
                exit(exitcode::IOERR)
            }
        }
    }

    pub fn get_searcher() -> Searcher {
        let index = Self::get_index();
        match index.reader() {
            Ok(reader) => reader.searcher(),
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
        let tokenizer = tantivy_jieba::JiebaTokenizer {};
        let index_path = &CONFIG_PATH.join("data");
        let result = if !index_path.exists() {
            // 如果这个目录不存在 Tantivy 就会报错,所以需要手动创建,文件夹里有没有索引倒是次要的
            FileUtils::create_dir(index_path);
            Index::create_in_dir(index_path, SONG_SCHEMA.clone())
        } else {
            Index::open_in_dir(index_path)
        };
        match result {
            Ok(index) => {
                index.tokenizers().register("jieba", tokenizer);
                index
            }
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

        Self::re_create_index();
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

    /// 删除原有的索引重新建立
    fn re_create_index() {
        if CONFIG_PATH.join("data").exists() {
            info!("删除原有的索引");
            FileUtils::delete_folder_contents(&CONFIG_PATH.join("data")).unwrap();
            fs::remove_dir(&CONFIG_PATH.join("data")).unwrap();
        }
    }

    /// 按照传入的 ID 查询歌曲,精确查询
    pub fn search_song_by_id(id: usize) -> Option<Song> {
        let searcher = Self::get_searcher();
        let query_parser =
            QueryParser::for_index(&Self::get_index(), vec![Song::field(SongField::Id)]);
        let query = query_parser.parse_query(id.to_string().as_str()).unwrap();

        let top_docs = match searcher.search(&query, &TopDocs::with_limit(1)) {
            Ok(top_docs) => top_docs,
            Err(error) => {
                error!("查询歌曲[{}]时出现错误\n[Cause]:{:?}", id, error);
                exit(exitcode::DATAERR)
            }
        };

        // ID 是唯一的,所以只会有一个结果
        return match top_docs.len() {
            1 => Some(
                match Song::from_document(&searcher.doc(top_docs[0].1).unwrap()) {
                    Ok(song) => song,
                    Err(error) => {
                        error!("反序列化错误\n[Cause]:{:?}", error);
                        exit(exitcode::IOERR)
                    }
                },
            ),
            _ => None,
        };
    }

    /// 按照 title 字段模糊查询歌曲
    pub fn search_songs_by_title(param: &str, count: usize) -> Vec<Song> {
        let mut query_parser =
            QueryParser::for_index(&Self::get_index(), vec![Song::field(SongField::Title)]);
        query_parser.set_field_fuzzy(Song::field(SongField::Title), false, 0, true);
        let searcher = Self::get_searcher();
        // 舞萌里一大堆繁体中文,优先查一下繁体
        let mut top_docs = Self::search_song(
            format!("{}", zhconv(param, Variant::ZhHant)).as_str(),
            count,
            &query_parser,
        );
        if top_docs.is_empty() {
            top_docs = Self::search_song(param, count, &query_parser);
        }
        return top_docs
            .iter()
            .map(|(_, doc)| searcher.doc(*doc).unwrap())
            .filter_map(|doc| Song::from_document(&doc).ok())
            .collect::<Vec<Song>>();
    }

    fn search_song(
        param: &str,
        count: usize,
        query_parser: &QueryParser,
    ) -> Vec<(Score, DocAddress)> {
        let searcher = Self::get_searcher();
        let query = query_parser.parse_query(param).unwrap();
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(count))
            .unwrap_or_else(|error| {
                error!("查询歌曲[{}]时出现错误\n[Cause]:{:?}", param, error);
                exit(exitcode::IOERR)
            });
        top_docs
    }
}
