use std::cmp::min;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

use crate::config::consts::{CONFIG_PATH, PROFILE};
use crate::db::database::MaimaiDB;
use crate::db::entity::Song;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{error, info, warn};
use reqwest::blocking::Response;
use zip::ZipArchive;

pub struct ResourceService {}

impl ResourceService {
    /// 更新谱面信息和下载静态文件
    pub fn update_songs_data() {
        let url = &PROFILE.remote_api.json_url;
        info!("正在从[{}]下载谱面信息", url);
        // 删除原有的表格重建会较快
        MaimaiDB::re_create_table();
        let songs = match reqwest::blocking::get(url) {
            Ok(response) => response.json::<Vec<Song>>(),
            Err(error) => {
                error!("获取服务器信息出错:{:?}", error);
                exit(exitcode::UNAVAILABLE)
            }
        }
        .unwrap();

        let progress_bar = ProgressBar::new(songs.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:50.green/white} 歌曲数量: {pos}/{len} [{elapsed_precise}]")
                .unwrap(),
        );

        let connection = MaimaiDB::get_connection();
        let mut statement = connection.prepare_cached("INSERT INTO songs (id, title, song_type, ds, level, cids, charts, basic_info) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)").expect("SQL 解析失败");
        for song in &songs {
            statement
                .execute(&[
                    &song.id,
                    &song.title,
                    &song.song_type,
                    &serde_json::to_string(&song.ds).unwrap(),
                    &serde_json::to_string(&song.level).unwrap(),
                    &serde_json::to_string(&song.cids).unwrap(),
                    &serde_json::to_string(&song.charts).unwrap(),
                    &serde_json::to_string(&song.basic_info).unwrap(),
                ])
                .unwrap();
            progress_bar.inc(1);
        }
        progress_bar.finish();
    }

    /// 获取资源文件并解压
    pub fn update_resource(force: bool) {
        // 默认的文件名为 static.zip
        let resource_zip = &CONFIG_PATH.join("static.zip");

        // 发起GET请求并获取响应
        let response = match reqwest::blocking::get(&PROFILE.remote_api.resource_url) {
            Ok(response) => {
                // 检查响应状态是否成功
                if !response.status().is_success() {
                    error!("下载文件时出现问题：{:?}", response.status());
                    exit(exitcode::IOERR)
                }
                response
            }
            Err(error) => {
                error!("无法连接到服务器,请检查网络连接:\n\t{:?}", error);
                exit(exitcode::UNAVAILABLE)
            }
        };

        Self::check_file(resource_zip, force, response);

        // 获取需要解压的文件
        let archive = File::open(resource_zip).unwrap();
        let mut zip = match ZipArchive::new(archive) {
            Ok(zip) => zip,
            Err(err) => {
                error!(
                    "无法解压资源文件,可以尝试使用 --force(-f) 参数进行强制更新\n\t{:?}",
                    err
                );
                exit(exitcode::IOERR)
            }
        };

        // 创建资源文件夹,如果存在则删除
        let resource_path = CONFIG_PATH.join("resource");
        if resource_path.exists() {
            fs::remove_dir_all(resource_path.as_path()).unwrap();
        }
        fs::create_dir_all(resource_path.as_path()).unwrap();

        Self::extract_zip_archive(&mut zip, resource_path);
        info!("资源文件解压成功");
    }

    /// 检查文件是否合法(例如: 文件大小不正确或不存在,这种情况多半是寄了，需要重新下载)
    ///
    /// 如果携带强制标识,删除资源文件重建
    fn check_file(resource_zip: &PathBuf, force: bool, response: Response) {
        if force && resource_zip.exists() {
            fs::remove_file(resource_zip).unwrap();
        }

        // 文件不存在开始下载
        if !resource_zip.exists() {
            // 下载文件
            Self::download_resource(resource_zip, response);
            info!("资源文件下载成功,开始解压资源文件...");
            return;
        }

        // 如果上面的下载逻辑成功，无论下没下完都能获得 metadata,拿到长度
        let content_length = match fs::metadata(resource_zip) {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                error!("无法获取下载文件详情\n\t{:?}", error);
                exit(exitcode::IOERR)
            }
        };

        // 这里处理文件长度,场景是下载了但没完全下完的时候，压缩包大小不对，也有可能是静态文件发生了变化,总之是要重下
        if !content_length.eq(&response.content_length().unwrap_or(0)) {
            warn!("资源文件已存在,但是文件大小不正确,开始重新下载...");
            fs::remove_file(resource_zip).unwrap();
            Self::download_resource(resource_zip, response);
            info!("资源文件下载成功,开始解压资源文件...");
            return;
        }
        info!("资源文件已存在,无需下载,开始解压资源文件...")
    }

    /// 解压 zip 文件
    fn extract_zip_archive(zip: &mut ZipArchive<File>, resource_path: PathBuf) {
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            // 只需要 mai/cover 文件夹下的谱面资源文件
            if !file.is_dir() && file.name().starts_with("mai/cover/") {
                let file_name = file.name();
                // 控制过滤文件夹,并将该路径截断,仅保留文件名
                let file_path = resource_path.join(Path::new(&file_name["mai/cover/".len()..]));
                let mut target_file = match file_path.exists() {
                    true => File::open(file_path).unwrap(),
                    false => File::create(file_path).unwrap(),
                };
                std::io::copy(&mut file, &mut target_file).unwrap();
            }
        }
    }

    /// 下载资源文件
    ///
    /// 资源文件路径可以在配置文件内配置
    fn download_resource(resource_zip: &PathBuf, response: Response) {
        info!("正在从[{}]下载资源文件", &PROFILE.remote_api.resource_url);

        let total_size = match response.content_length() {
            None => {
                error!("下载文件时出现问题,获取的文件大小为 0");
                exit(exitcode::IOERR)
            }
            Some(size) => size,
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
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bar:50.green/white} 下载进度: {bytes}/{total_bytes} [ETA: {eta}]")
                .unwrap()
                .with_key(
                    "eta",
                    |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                        write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                    },
                ),
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
                    error!("文件写入出现问题:{:?}", error);
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
