#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;

use clap::Parser;
use log::{error, warn};
use platform_dirs::AppDirs;
use prettytable::{Attr, Cell};
use prettytable::color::{GREEN, MAGENTA, RED, WHITE, YELLOW};
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator, TableFormat};

use crate::client::DXProberClient;
use crate::database::MaimaiDB;
use crate::printer_handler::PrinterHandler;
use crate::profiles::Profile;

mod client;
mod database;
mod printer_handler;
mod profiles;
mod simple_log;

lazy_static! {
    // 在 MacOS下遵守 XDG 规范,即创建的配置文件夹为 `~/.config/maimai-search`
    static ref CONFIG_PATH: PathBuf = AppDirs::new(Some("maimai-search"), true).unwrap().config_dir;
    static ref PROFILE: Profile = profiles::Profile::new();
    static ref DIFFICULT_NAME: Vec<Cell> = vec!["BASIC", "ADVANCED", "EXPERT", "MASTER", "Re:MASTER"].iter()
        .zip(&[GREEN, YELLOW, RED, MAGENTA, WHITE])
        .map(|(difficult, column_color)| Cell::new(difficult).with_style(Attr::ForegroundColor(*column_color)))
        .collect();
    static ref LAUNCH_PATH: PathBuf = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    static ref MARKDOWN_TABLE_STYLE: TableFormat = FormatBuilder::new()
        .column_separator('|').borders('|')
        .separators(&[LinePosition::Title], LineSeparator::new('-', '|', '|', '|'))
        .padding(1, 1).build();
}

/// GitHub Repository : [https://github.com/Anselyuki/maimai-search-rs]
#[derive(Parser, Debug)]
#[command(name = "maimai-search", bin_name = "maimai-search")]
#[command(author, about, version, next_line_help = false)]
struct Args {
    // 子命令枚举
    #[command(subcommand)]
    command: Option<SubCommands>,
    /// 检索信息,如果打不出片假名没有关系,可以试试只把中文打进去(君の日本语本当上手)
    name: Option<String>,
    /// 模糊查询的匹配数量(由于实现比较简陋,往后的匹配结果可能会过于离谱)
    #[arg(short, long, default_value = "3")]
    count: usize,
    /// 开启详情查询
    #[arg(short, long)]
    detail: bool,
    /// 使用 markdown 格式输出
    #[arg(short, long)]
    markdown: bool,
    /// 指定 markdown 输出的文件名称(路径使用当前程序执行的路径)
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// 更新谱面信息数据库
    Update {
        /// 强制更新
        #[arg(short, long)]
        force: bool,
    },
    ///  使用 ID 进行检索，如：maimai-search id 11571 11524
    Id {
        /// 检索 ID ,支持多个 ID 检索
        ids: Vec<usize>,
        /// 使用 markdown 格式输出
        #[arg(short, long)]
        markdown: bool,
        /// 指定 markdown 输出的文件名称(路径使用当前程序执行的路径)
        #[arg(short, long)]
        output: Option<String>,
        /// 开启详情查询
        #[arg(short, long)]
        detail: bool,
    },
    /// 配置文件管理,详情请运行 maimai-search config --help
    Config {
        /// 在配置文件夹内创建默认配置文件
        #[arg(short, long)]
        default: bool,
    },
}

fn main() {
    simple_log::init().unwrap();
    let args = Args::parse();
    MaimaiDB::init();

    // 主要处理命令触发的逻辑
    match args.command {
        Some(SubCommands::Update { force }) => DXProberClient::update_data(&PROFILE.remote.json_url, force),
        Some(SubCommands::Id { ids, markdown, output, detail }) => {
            let mut songs = vec![];
            for id in ids {
                let results = DXProberClient::search_songs_by_id(id);
                for song in results {
                    songs.push(song);
                }
            }
            PrinterHandler::new(songs, detail, markdown, output);
        }
        Some(SubCommands::Config { default }) => if default { Profile::create_default() },
        // 子命令为空时,表示使用主功能: 按照名称查询
        None => match args.name {
            Some(name) => {
                let songs = DXProberClient::search_songs_by_name(name.as_str(), args.count);
                PrinterHandler::new(songs, args.detail, args.markdown, args.output);
            }
            None => {
                get_exist_args(&args);
                error!("[NAME] 参数为空,请使用 --help 或者 -h 查看详情");
            }
        },
    }
}

/// 定义需要处理的字段和对应的字符串表示
fn get_exist_args(args: &Args) {
    let fields_to_collect = [
        (args.detail, "detail"), (args.markdown, "markdown"),
        (args.output.is_some(), &format!("output = {}", args.output.clone().unwrap_or("".to_string()))),
    ];
    let collected_args: Vec<_> = fields_to_collect.iter()
        .filter(|(flag, _)| *flag)
        .map(|(_, name)| name).collect();
    if !collected_args.is_empty() {
        warn!("检测到不能单独使用的参数: {:?}",  collected_args);
    }
}