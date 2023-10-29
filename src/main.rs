#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use platform_dirs::AppDirs;

use crate::client::DXProberClient;
use crate::database::MaimaiDB;
use crate::entity::Song;
use crate::printer::Printer;

mod client;
mod entity;
mod database;
mod printer;

lazy_static! {
    // 在 MacOS下遵守 XDG 规范,即创建的配置文件夹为 `~/.config/maimai-search`
    static ref CONFIG_PATH: PathBuf = AppDirs::new(Some("maimai-search"), true).unwrap().config_dir;
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, default_value = "https://www.diving-fish.com/api/maimaidxprober/music_data")]
    url: String,
}


#[derive(Subcommand, Debug)]
enum Commands {
    /// 搜索谱面信息,如果同时传入 id 参数与 name 参数,将优先使用 id 进行精确查询
    Search {
        /// 根据名称搜索 (模糊检索)
        #[arg(short, long)]
        name: Option<String>,
        /// 根据歌曲 ID 搜索 (精确)
        #[arg(short, long)]
        id: Option<String>,
        /// 是否开启 markdown 输出
        #[arg(short, long)]
        md: bool,
        /// 是否开启详情查询
        #[arg(short, long)]
        detail: bool,
    },
    /// 更新谱面信息数据库
    Update {},
}

fn main() {
    let args = Args::parse();
    MaimaiDB::init();

    match args.command {
        Some(Commands::Update {}) => {
            DXProberClient::update_data(args.url);
        }
        Some(Commands::Search { name, id, md, detail }) => {
            let songs = search(name, id);
            Printer::print_detail(songs.clone());
        }
        _ => {}
    }
}

/// 搜索谱面信息,如果同时传入 id 参数与 name 参数,将优先使用 id 进行精确查询
fn search(name: Option<String>, id: Option<String>) -> Vec<Song> {
// 根据名称搜索 (模糊检索)
    let mut songs = Vec::new();
    if let Some(id) = id {
        match DXProberClient::search_songs_by_id(id.as_str()) {
            Some(song) => songs.push(song),
            _ => {}
        }
    } else if let Some(name) = name {
        songs = DXProberClient::search_songs_by_name(name.as_str());
    } else {
        println!("搜索必须指定 --id(-i) 或者 --name(-n) 参数");
    }

    if songs.is_empty() {
        println!("未找到歌曲,可以尝试使用 --name 参数进行模糊搜索或者使用 update 更新数据库");
        exit(1);
    }
    songs
}

