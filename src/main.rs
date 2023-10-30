#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;

use clap::Parser;
use platform_dirs::AppDirs;
use prettytable::Attr;
use prettytable::Cell;
use prettytable::color;

use crate::client::DXProberClient;
use crate::database::MaimaiDB;
use crate::printer::Printer;
use crate::profiles::Profile;

mod client;
mod database;
mod printer;
mod profiles;

lazy_static! {
    // 在 MacOS下遵守 XDG 规范,即创建的配置文件夹为 `~/.config/maimai-search`
    static ref CONFIG_PATH: PathBuf = AppDirs::new(Some("maimai-search"), true).unwrap().config_dir;
    static ref PROFILE: Profile = profiles::Profile::new();
    static ref DIFFICULT_NAME: Vec<Cell> = vec!["BASIC", "ADVANCED", "EXPERT", "MASTER", "Re:MASTER"].iter()
        .zip(&[color::GREEN, color::YELLOW, color::RED, color::BRIGHT_MAGENTA, color::BRIGHT_WHITE])
        .map(|(difficult, column_color)| Cell::new(difficult).with_style(Attr::ForegroundColor(*column_color)))
        .collect();
    static ref LAUNCH_PATH: PathBuf = std::env::current_exe().unwrap();
}

/// GitHub Repository : [https://github.com/Anselyuki/maimai-search-rs]
#[derive(Parser, Debug)]
#[command(name = "maimai-search", bin_name = "maimai-search")]
#[command(author, about, version, next_line_help = false)]
struct Args {
    /// 检索信息(使用 --id 参数时为 id);
    /// 如果打不出片假名没有关系,可以试试只把中文打进去(君の日本语本当上手)
    name: Option<String>,

    /// 模糊查询的匹配数量(由于实现比较简陋,往后的匹配结果可能会过于离谱)
    #[arg(short, long, default_value = "3")]
    count: usize,
    /// 开启详情查询
    #[arg(short, long)]
    detail: bool,
    /// 使用 id 检索歌曲,使用 id 检索歌曲自动开启详情查询
    #[arg(short, long)]
    id: bool,
    /// 使用 markdown 格式输出
    #[arg(short, long)]
    md: bool,

    // 子命令枚举
    #[command(subcommand)]
    command: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// 更新谱面信息数据库
    Update {},
}

fn main() {
    let args = Args::parse();
    MaimaiDB::init();

    match args.name {
        Some(name) => match args.id {
            true => match DXProberClient::search_songs_by_id(name.as_str()) {
                Some(song) => Printer::print_song_detail_single(song),
                None => println!(
                    "未找到歌曲,可以尝试添加更多的关键字或者使用 update 更新数据库"
                ),
            },
            false => {
                let songs = DXProberClient::search_songs_by_name(name.as_str(), args.count);
                if args.detail {
                    Printer::print_songs_detail_multi(songs);
                } else {
                    Printer::print_songs_info(songs);
                }
            }
        },
        None => match args.command {
            Some(SubCommands::Update {}) => {
                DXProberClient::update_data(&PROFILE.url);
            }
            None => {
                println!("参数不能为空!");
            }
        },
    }
}
