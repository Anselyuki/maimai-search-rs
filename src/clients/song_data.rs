use std::process::exit;

use log::{error, info, warn};

use crate::clients::song_data::entity::Song;
use crate::config::consts::PROFILE;
use crate::db::database::MaimaiDB;

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
pub fn search_songs_by_title(param: &str, count: usize) -> Vec<Song> {
    let songs = MaimaiDB::search_songs_by_title(param.to_lowercase().as_str(), count);
    if songs.is_empty() {
        warn!(
            "查询参数[{}]找不到对应的歌曲!请尝试给出更多关键字或者更新数据",
            param
        );
        exit(exitcode::OK);
    }
    songs
}

pub mod entity {
    use std::io::Error;
    use std::process::exit;

    use log::error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tantivy::schema::{
        Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED,
    };
    use tantivy::{doc, Document};

    use crate::config::consts::SONG_SCHEMA;

    /// 歌曲
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Song {
        /// 歌曲 ID
        #[serde(serialize_with = "serialize_usize_as_string")]
        #[serde(deserialize_with = "deserialize_usize_from_string")]
        pub id: usize,
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

    /// # 序列化 usize 为 String
    ///
    /// 从远程获取的数据中,歌曲 ID 为字符串,本函数用于序列化 ID 字段
    fn serialize_usize_as_string<S>(id: &usize, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&id.to_string())
    }

    /// # 反序列化 String 为 usize
    ///
    /// 从远程获取的数据中,歌曲 ID 为字符串,本函数用于反序列化 ID 字段
    fn deserialize_usize_from_string<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id_str: String = Deserialize::deserialize(deserializer)?;
        id_str
            .parse()
            .map_err(|_| serde::de::Error::custom("expected a string containing a number"))
    }

    /// 谱面信息
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Chart {
        /// Note 数量分布
        pub notes: Vec<u32>,
        /// 谱面作者
        pub charter: String,
    }

    /// 歌曲基本信息
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

    /// 歌曲字段枚举,主要添加 Keywords 对 Tantivy 定制查询提供方便
    #[derive(PartialEq, strum_macros::Display)]
    pub enum SongField {
        #[strum(serialize = "id")]
        Id,
        #[strum(serialize = "keyword")]
        Keyword,
        #[strum(serialize = "title")]
        Title,
        #[strum(serialize = "song_type")]
        SongType,
        #[strum(serialize = "ds")]
        Ds,
        #[strum(serialize = "level")]
        Level,
        #[strum(serialize = "cids")]
        Cids,
        #[strum(serialize = "charts")]
        Charts,
        #[strum(serialize = "basic_info")]
        BasicInfo,
    }

    impl Song {
        /// 获取 Tantivy 的 schema
        pub fn init_schema() -> Schema {
            let mut schema_builder = Schema::builder();
            // 被索引的字段为 id 与 title
            schema_builder.add_u64_field("id", INDEXED | FAST | STORED);
            let text_field_indexing = TextFieldIndexing::default()
                .set_tokenizer("jieba")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions);
            let text_field = TextOptions::default().set_indexing_options(text_field_indexing);
            schema_builder.add_text_field("keyword", text_field | STORED | FAST);
            schema_builder.add_text_field("title", STORED);
            // 其余的字段为存储字段,不被索引
            schema_builder.add_text_field("song_type", STORED);
            schema_builder.add_text_field("ds", STORED);
            schema_builder.add_text_field("level", STORED);
            schema_builder.add_text_field("cids", STORED);
            schema_builder.add_text_field("charts", STORED);
            schema_builder.add_text_field("basic_info", STORED);
            schema_builder.build()
        }

        /// 获得当前歌曲的文档类
        pub fn document(&self) -> Result<Document, serde_json::Error> {
            let doc = doc!(
                Self::field(SongField::Id) => self.id as u64,
                Self::field(SongField::Keyword) => self.title.to_lowercase(),
                Self::field(SongField::Title) => &*self.title,
                Self::field(SongField::SongType) => &*self.song_type,
                Self::field(SongField::Ds) => serde_json::to_string(&self.ds)?,
                Self::field(SongField::Level) => serde_json::to_string(&self.level)?,
                Self::field(SongField::Cids) => serde_json::to_string(&self.cids)?,
                Self::field(SongField::Charts) => serde_json::to_string(&self.charts)?,
                Self::field(SongField::BasicInfo) => serde_json::to_string(&self.basic_info)?,
            );
            Ok(doc)
        }

        /// 单独获取字段(静态方法)
        pub fn field(song_field: SongField) -> Field {
            match SONG_SCHEMA.get_field(&*song_field.to_string()) {
                Ok(field) => field,
                Err(error) => {
                    error!("获取 Field 失败\n[Cause]:{:?}", error);
                    exit(exitcode::DATAERR);
                }
            }
        }

        /// 从文档类转换为实体类(反序列化)
        pub fn from_document(retrieved_doc: &Document) -> Result<Song, Error> {
            macro_rules! deserialize_field {
                ($doc:expr, $field:expr, $type:ty) => {
                    serde_json::from_str::<$type>(
                        get_field!($doc, $field)
                            .as_text()
                            .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?,
                    )
                    .expect("反序列化失败")
                };
            }
            macro_rules! get_field {
                ($retrieved_doc:expr, $field:expr) => {
                    $retrieved_doc
                        .get_first(SONG_SCHEMA.get_field($field).expect("获取字段失败"))
                        .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                };
            }
            Ok(Song {
                id: get_field!(retrieved_doc, "id")
                    .as_u64()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?
                    as usize,
                title: get_field!(retrieved_doc, "title")
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?
                    .to_string(),
                song_type: get_field!(retrieved_doc, "song_type")
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?
                    .to_string(),
                ds: deserialize_field!(retrieved_doc, "ds", Vec<f32>),
                level: deserialize_field!(retrieved_doc, "level", Vec<String>),
                cids: deserialize_field!(retrieved_doc, "cids", Vec<u32>),
                charts: deserialize_field!(retrieved_doc, "charts", Vec<Chart>),
                basic_info: deserialize_field!(retrieved_doc, "basic_info", BasicInfo),
            })
        }
    }
}
