use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use jieba_rs::Jieba;
use levenshtein::levenshtein;
use log::{error, info, warn};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use crate::{CONFIG_PATH, PROFILE};
use crate::database::MaimaiDB;

pub struct DXProberClient {}

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

// 用于从服务器更新谱面信息
impl DXProberClient {
    /// 更新谱面信息和下载静态文件
    pub fn update_data() {
        let url = &PROFILE.remote.json_url;
        info!("正在从[{}]下载谱面信息", url);
        // 删除原有的表格重建会较快
        MaimaiDB::re_create_table();
        let songs = match reqwest::blocking::get(url) {
            Ok(response) => { response.json::<Vec<Song>>() }
            Err(error) => {
                error!("获取服务器信息出错:{:?}", error);
                exit(exitcode::UNAVAILABLE)
            }
        }.unwrap();

        let progress_bar = ProgressBar::new(songs.len() as u64);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("{bar:50.green/white} 歌曲数量: {pos}/{len} [{elapsed_precise}]").unwrap()
        );

        let connection = MaimaiDB::get_connection();
        let mut statement = connection.prepare_cached("INSERT INTO songs (id, title, song_type, ds, level, cids, charts, basic_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)").expect("SQL 解析失败");
        for song in &songs {
            statement.execute(&[
                &song.id,
                &song.title,
                &song.song_type,
                &serde_json::to_string(&song.ds).unwrap(),
                &serde_json::to_string(&song.level).unwrap(),
                &serde_json::to_string(&song.cids).unwrap(),
                &serde_json::to_string(&song.charts).unwrap(),
                &serde_json::to_string(&song.basic_info).unwrap()
            ]).unwrap();
            progress_bar.inc(1);
        }
        progress_bar.finish();
    }

    /// 按照 id 查询歌曲
    pub fn search_songs_by_id(id: usize) -> Vec<Song> {
        let sql = format!("SELECT id, title, song_type, ds, level, cids, charts, basic_info from songs where id = {};", id);
        match MaimaiDB::search_song(sql) {
            None => { Vec::new() }
            Some(song) => { vec![song] }
        }
    }

    /// 按照名称查询歌曲
    pub fn search_songs_by_name(name: &str, count: usize) -> Vec<Song> {
        let stop_words: HashSet<String> = ["的", " ", "!", "\"", "“", "”", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "=", "+", "[", "]", "{", "}", ";", ":", "<", ">", ",", ".", "/", "?"].iter().map(|&s| s.to_string()).collect();
        let keywords: Vec<String> = Jieba::new().cut(name, true).iter()
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
    fn similar_list_top(partial_song: HashMap<String, Song>, name: &str, count: usize) -> Vec<Song> {
        // 计算 Levenshtein 距离，并排序
        let mut songs: Vec<(usize, Song)> = partial_song.iter()
            .map(|(_, song)| { (levenshtein(name, &*song.title), song.clone()) })
            .filter(|tuple| (tuple.0 < 100)).collect();
        songs.sort_by(|a, b| a.0.cmp(&b.0));
        // 选择前5个匹配项
        songs.into_iter()
            .take(count)
            .map(|(_, song)| song)
            .collect()
    }

    /// 获取资源文件并解压
    pub fn update_resource(force: bool) {
        // 默认的文件名为 static.zip
        let resource_zip = &CONFIG_PATH.join("static.zip");
        let client = reqwest::blocking::Client::new();

        // 发起GET请求并获取响应
        let response = match client.get(&PROFILE.remote.resource_url).send() {
            Ok(response) => { response }
            Err(_) => {
                error!("无法连接到服务器,请检查网络连接");
                exit(exitcode::UNAVAILABLE)
            }
        };

        // 检查响应状态是否成功
        if !response.status().is_success() {
            error!("下载文件时出现问题：{:?}", response.status());
            exit(exitcode::IOERR)
        }

        // 携带强制标识,删除资源文件重建
        if force && resource_zip.exists() { fs::remove_file(resource_zip.as_path()).unwrap(); }

        if !resource_zip.exists() {
            // 下载文件
            Self::download_resource(resource_zip, response);
            info!("资源文件下载成功,开始解压资源文件...");
        } else {
            info!("资源文件已存在,无需下载,开始解压资源文件...");
        }

        // 获取需要解压的文件
        let archive = File::open(resource_zip).unwrap();
        let mut zip = match ZipArchive::new(archive) {
            Ok(zip) => zip,
            Err(err) => {
                error!("无法解压资源文件,可以尝试使用 --force(-f) 参数进行强制更新\n\t{:?}", err);
                exit(exitcode::IOERR)
            }
        };

        // 创建资源文件夹,如果存在则删除
        let resource_path = CONFIG_PATH.join("resource");
        if resource_path.exists() { fs::remove_dir_all(resource_path.as_path()).unwrap(); }
        fs::create_dir_all(resource_path.as_path()).unwrap();

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            // 只需要 mai/cover 文件夹下的谱面资源文件
            if !file.is_dir() && file.name().starts_with("mai/cover/") {
                let file_name = file.name();
                // 控制过滤文件夹,并将该路径截断,仅保留文件名
                let file_path = resource_path.join(Path::new(&file_name["mai/cover/".len()..]));
                let mut target_file = match file_path.exists() {
                    true => File::open(file_path).unwrap(),
                    false => File::create(file_path).unwrap()
                };
                std::io::copy(&mut file, &mut target_file).unwrap();
            }
        }
        info!("资源文件解压成功");
    }

    /// 下载资源文件
    ///
    /// 资源文件路径可以在配置文件内配置
    fn download_resource(resource_zip: &PathBuf, response: Response) {
        info!("正在从[{}]下载资源文件", &PROFILE.remote.resource_url);

        let total_size = match response.content_length() {
            None => {
                error!("下载文件时出现问题,获取的文件大小为 0");
                exit(exitcode::IOERR)
            }
            Some(size) => size
        };

        // 创建文件来保存下载的内容
        let mut zip_file = match File::create(resource_zip) {
            Ok(file) => file,
            Err(error) => {
                error!("创建文件出现问题:{:?}", error);
                exit(exitcode::IOERR)
            }
        };
        // 从响应中读取ZIP内容并写入文件
        let mut reader = BufReader::new(response);
        let mut buffer = [0; 4096];

        let progress_bar = ProgressBar::new(total_size);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("{bar:50.green/white} 下载进度: {bytes}/{total_bytes} [ETA: {eta}]").unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        );
        let mut downloaded: u64 = 0;
        loop {
            let bytes_read = match reader.read(&mut buffer) {
                Ok(read) => read,
                Err(error) => {
                    error!("下载文件时出现问题:\n\t{:?}", error);
                    exit(exitcode::IOERR)
                }
            };
            if bytes_read == 0 {
                break;
            }
            match zip_file.write_all(&buffer[0..bytes_read]) {
                Err(error) => {
                    error!("文件写入出现问题:{:?}",error);
                    exit(exitcode::IOERR)
                }
                _ => {}
            }
            downloaded = min(downloaded + bytes_read as u64, total_size);
            progress_bar.set_position(downloaded);
        }
        progress_bar.finish();
    }
}