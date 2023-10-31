use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use jieba_rs::Jieba;
use levenshtein::levenshtein;
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
    /// 更新谱面信息，删除表重新建比较快
    pub fn update_data(url: &String, force: bool) {
        println!("{}: 正在从[{}]下载谱面信息", "info".green().bold(), url);
        MaimaiDB::re_create_table();

        let songs = match reqwest::blocking::get(url) {
            Ok(response) => { response.json::<Vec<Song>>() }
            Err(error) => panic!("获取服务器信息出错:{:?}", error)
        }.unwrap();

        let progress_bar = ProgressBar::new(songs.len() as u64);

        progress_bar.set_style(
            ProgressStyle::default_bar()
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
        DXProberClient::get_static(force);
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
            println!("{}: 查询关键字[{}]找不到匹配项", "warning".red().bold(), name);
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

    fn get_static(force: bool) {
        let url = &PROFILE.resource_url;
        let resource_zip = &CONFIG_PATH.join("static.zip");
        let client = reqwest::blocking::Client::new();

        // 发起GET请求并获取响应
        let response = match client.get(url).send() {
            Ok(response) => { response }
            Err(_) => {
                eprintln!("{}: 无法连接到服务器,请检查网络连接", "error".red().bold());
                exit(exitcode::NOHOST)
            }
        };

        // 检查响应状态是否成功
        if !response.status().is_success() {
            eprintln!("{}: 下载文件时出现问题：{:?}", "error".red().bold(), response.status());
            exit(exitcode::IOERR)
        }

        if force && resource_zip.exists() { fs::remove_file(resource_zip.as_path()).unwrap(); }

        if !resource_zip.exists() || (resource_zip.exists() && fs::metadata(resource_zip).unwrap().len() < 90000000) {
            // 下载文件
            Self::download_resource(resource_zip, response);
            print!("{}: 资源文件下载成功,开始解压资源文件...", "info".green().bold());
        } else {
            print!("{}: 资源文件已存在,无需下载,开始解压资源文件...", "info".green().bold());
        }

        // 获取需要解压的文件
        let archive = File::open(resource_zip).unwrap();
        let mut zip = match ZipArchive::new(archive) {
            Ok(zip) => zip,
            Err(err) => {
                eprintln!("无法解压ZIP文件：{:?}" ,err);
                exit(exitcode::IOERR)
            }
        };

        // 创建资源文件夹,如果存在则删除
        let resource_path = CONFIG_PATH.join("resource");
        if resource_path.exists() { fs::remove_dir_all(resource_path.as_path()).unwrap(); }
        fs::create_dir_all(resource_path.as_path()).unwrap();

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();

            let prefix = "mai/cover/";
            if !file.name().starts_with(prefix) {
                continue;
            }

            if file.is_dir() {
                let target = resource_path.join(Path::new(&file.name().replace("\\", "")));
                fs::create_dir_all(target).unwrap();
            } else {
                let file_name = file.name();
                let stripped = &file_name[prefix.len()..];
                let file_path = resource_path.join(Path::new(stripped));
                let mut target_file = if !file_path.exists() {
                    File::create(file_path).unwrap()
                } else {
                    File::open(file_path).unwrap()
                };
                std::io::copy(&mut file, &mut target_file).unwrap();
            }
        }
        println!("资源文件解压成功");
    }

    fn download_resource(resource_zip: &PathBuf, response: Response) {
        println!("\n{}: 正在从[{}]下载资源文件", "info".green().bold(), &PROFILE.resource_url);

        let total_size = response.content_length().unwrap_or(0);

        // 创建一个文件来保存下载的内容
        let mut zip_file = File::create(resource_zip).unwrap();
        // 从响应中读取ZIP内容并写入文件
        let mut reader = BufReader::new(response);
        let mut buffer = [0; 4096];

        let pb = ProgressBar::new(total_size);

        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:50.green/white} 下载进度: {bytes}/{total_bytes} [ETA: {eta}]").unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        );


        // pb.set_style(ProgressStyle::with_template("{bar:75.green/white} {bytes}/{total_bytes} [{elapsed_precise}]").unwrap()
        //     .progress_chars("█-"));

        let mut downloaded: u64 = 0;

        loop {
            let bytes_read = reader.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            zip_file.write_all(&buffer[0..bytes_read]).unwrap();

            let new = min(downloaded + bytes_read as u64, total_size);
            downloaded = new;
            pb.set_position(new);
        }
        pb.finish();
    }
}