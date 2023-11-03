use serde::{Deserialize, Serialize};
use tantivy::schema::{Schema, STORED, TEXT};

/// 歌曲
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    /// 歌曲 ID
    pub id: String,
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

impl Song {
    pub(crate) fn get_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", TEXT | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("song_type", TEXT | STORED);
        schema_builder.add_u64_field("ds", STORED);
        schema_builder.add_u64_field("level", STORED);
        schema_builder.add_facet_field("cids", ());
        schema_builder.add_facet_field("charts", ());
        schema_builder.add_text_field("basic_info", TEXT | STORED);
        schema_builder.build()
    }
}
