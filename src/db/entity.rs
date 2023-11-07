use std::fmt::Debug;
use std::io::Error;

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

fn serialize_usize_as_string<S>(id: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&id.to_string())
}

/// 序列化 usize 为字符串
///
/// 从远程获取的数据中,歌曲 ID 为字符串
///
/// 但是这个字段全部都是正整数类型,故在本地索引中,序列化歌曲 ID 为 usize
fn deserialize_usize_from_string<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let id_str: String = Deserialize::deserialize(deserializer)?;
    id_str
        .parse()
        .map_err(|_| serde::de::Error::custom("expected a string containing a number"))
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

#[derive(PartialEq)]
pub enum SongField {
    Id,
    Keyword,
    Title,
    SongType,
    Ds,
    Level,
    Cids,
    Charts,
    BasicInfo,
}

impl SongField {
    pub fn to_string(&self) -> &str {
        match self {
            SongField::Id => "id",
            SongField::Keyword => "keyword",
            SongField::Title => "title",
            SongField::SongType => "song_type",
            SongField::Ds => "ds",
            SongField::Level => "level",
            SongField::Cids => "cids",
            SongField::Charts => "charts",
            SongField::BasicInfo => "basic_info",
        }
    }
}

impl Song {
    /// 获取 Tantivy 的 schema 与所有的字段列
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

    pub fn document(&self) -> Result<Document, serde_json::Error> {
        let doc = doc!(
            Self::field(SongField::Id) => self.id.clone() as u64,
            Self::field(SongField::Keyword) => &*self.title.to_lowercase(),
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
        SONG_SCHEMA.get_field(song_field.to_string()).unwrap()
    }

    pub fn from_document(retrieved_doc: &Document) -> Result<Song, Error> {
        let schema = SONG_SCHEMA.clone();
        Ok(Song {
            id: retrieved_doc
                .get_first(schema.get_field("id").expect("获取字段失败"))
                .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                .as_u64()
                .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?
                as usize,
            title: retrieved_doc
                .get_first(schema.get_field("title").expect("获取字段失败"))
                .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                .as_text()
                .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?
                .to_string(),
            song_type: retrieved_doc
                .get_first(schema.get_field("song_type").expect("获取字段失败"))
                .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                .as_text()
                .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?
                .to_string(),
            ds: serde_json::from_str::<Vec<f32>>(
                retrieved_doc
                    .get_first(schema.get_field("ds").expect("获取字段失败"))
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?,
            )
            .expect("反序列化失败"),
            level: serde_json::from_str::<Vec<String>>(
                retrieved_doc
                    .get_first(schema.get_field("level").expect("获取字段失败"))
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?,
            )
            .expect("反序列化失败"),
            cids: serde_json::from_str::<Vec<u32>>(
                retrieved_doc
                    .get_first(schema.get_field("cids").expect("获取字段失败"))
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?,
            )
            .expect("反序列化失败"),
            charts: serde_json::from_str::<Vec<Chart>>(
                retrieved_doc
                    .get_first(schema.get_field("charts").expect("获取字段失败"))
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?,
            )
            .expect("反序列化失败"),
            basic_info: serde_json::from_str::<BasicInfo>(
                retrieved_doc
                    .get_first(schema.get_field("basic_info").expect("获取字段失败"))
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段不存在"))?
                    .as_text()
                    .ok_or(Error::new(std::io::ErrorKind::NotFound, "字段值为空"))?,
            )
            .expect("反序列化失败"),
        })
    }
}
