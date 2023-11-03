#[macro_use]
extern crate clap;

use std::process::exit;

use crate::config::profiles::Profile;
use crate::db::database::MaimaiDB;
use crate::db::entity::Song;
use crate::service::client::DXProberClient;
use crate::service::resource::ResourceService;
use crate::utils::printer_handler::PrinterHandler;
use crate::utils::simple_log;
use clap::Parser;
use log::error;

pub mod config;
pub mod db;
pub mod service;
pub mod utils;

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
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    ///  使用 ID 进行检索，如：maimai-search id 11571 11524
    Id {
        /// 检索 ID ,支持多个 ID 检索
        ids: Vec<usize>,
        /// 开启详情查询
        #[arg(short, long)]
        detail: bool,
    },
    /// 使用 markdown 格式输出
    Md {
        #[command(subcommand)]
        command: Option<MarkdownSubCommands>,
        /// 检索信息,如果打不出片假名没有关系,可以试试只把中文打进去(君の日本语本当上手)
        name: Option<String>,
        /// 模糊查询的匹配数量(由于实现比较简陋,往后的匹配结果可能会过于离谱)
        #[arg(short, long, default_value = "3")]
        count: usize,
        /// 开启详情查询
        #[arg(short, long)]
        detail: bool,
        /// 指定 markdown 输出的文件名称(路径使用当前程序执行的路径)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 更新谱面信息数据库
    Update {},
    /// 更新资源文件
    Resource {
        /// 强制更新资源文件
        #[arg(short, long)]
        force: bool,
    },
    /// 配置文件管理,详情请运行 maimai-search config --help
    Config {
        /// 在配置文件夹内创建默认配置文件
        #[arg(short, long)]
        default: bool,
    },
}

/// 使用 markdown 格式输出
#[derive(Subcommand, Debug)]
enum MarkdownSubCommands {
    Id {
        /// 检索 ID ,支持多个 ID 检索
        ids: Vec<usize>,
        /// 指定 markdown 输出的文件名称(路径使用当前程序执行的路径)
        #[arg(short, long)]
        output: Option<String>,
        /// 开启详情查询
        #[arg(short, long)]
        detail: bool,
    },
}

fn main() {
    simple_log::init().unwrap();
    let args = Args::parse();
    MaimaiDB::init();

    // 主要处理命令触发的逻辑
    match args.command {
        // 子命令为空时,表示使用主功能: 按照名称查询
        None => {
            if let Some(name) = args.name {
                let songs = DXProberClient::search_songs_by_name(name.as_str(), args.count);
                PrinterHandler::new(songs, args.detail, false, None);
            } else {
                error_handler();
            }
        }
        // ID 检索子命令
        Some(SubCommands::Id { ids, detail }) => {
            let songs = ids
                .iter()
                .flat_map(|id| DXProberClient::search_songs_by_id(*id))
                .collect::<Vec<Song>>();
            PrinterHandler::new(songs, detail, false, None);
        }
        // 更新数据库子命令
        Some(SubCommands::Update {}) => ResourceService::update_songs_data(),
        // 更新资源文件子命令
        Some(SubCommands::Resource { force }) => ResourceService::update_resource(force),
        // 配置文件管理子命令
        Some(SubCommands::Config { default }) => {
            if default {
                Profile::create_default()
            }
        }
        // markdown 输出子命令
        Some(SubCommands::Md {
            command,
            name,
            count,
            detail,
            output,
        }) => match command {
            None => {
                if let Some(name) = name {
                    let songs = DXProberClient::search_songs_by_name(name.as_str(), count);
                    PrinterHandler::new(songs, detail, true, output);
                } else {
                    error_handler();
                }
            }
            Some(MarkdownSubCommands::Id {
                ids,
                output,
                detail,
            }) => {
                let songs = ids
                    .iter()
                    .flat_map(|id| DXProberClient::search_songs_by_id(*id))
                    .collect::<Vec<Song>>();
                PrinterHandler::new(songs, detail, true, output);
            }
        },
    }
}

/// 定义需要处理的字段和对应的字符串表示
fn error_handler() {
    error!("参数错误,请使用 --help 或者 -h 查看详情");
    exit(exitcode::USAGE)
}
