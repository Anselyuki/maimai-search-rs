use serde::{Deserialize, Serialize};

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
