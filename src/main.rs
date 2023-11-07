extern crate clap;

use std::process::exit;

use clap::Parser;
use log::error;
use maimai_search_lib::config::command::{MaimaiSearchArgs, MarkdownSubCommands, SubCommands};
use maimai_search_lib::config::profiles::Profile;
use maimai_search_lib::service::client::DXProberClient;
use maimai_search_lib::service::resource::ResourceService;
use maimai_search_lib::utils::printer::PrinterHandler;
use maimai_search_lib::utils::simple_log;

fn main() {
    simple_log::init().unwrap();
    let args = MaimaiSearchArgs::parse();

    // 主要处理命令触发的逻辑
    match args.command {
        // 子命令为空时,表示使用主功能: 按照名称查询
        None => {
            if let Some(name) = args.name {
                let songs = DXProberClient::search_songs_by_title(name.as_str(), args.count);
                PrinterHandler::console_handler(songs, args.detail, args.level);
            } else {
                error_handler();
            }
        }
        // ID 检索子命令
        Some(SubCommands::Id { ids, detail, level }) => {
            let songs = ids
                .iter()
                .flat_map(|id| DXProberClient::search_songs_by_id(*id))
                .collect();
            PrinterHandler::console_handler(songs, detail, level);
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
            add,
            level,
        }) => {
            if output.is_some() && add.is_some() {
                error!("add 参数和 output 参数不能同时使用");
                exit(exitcode::USAGE)
            }
            match command {
                None => {
                    if let Some(name) = name {
                        let songs = DXProberClient::search_songs_by_title(name.as_str(), count);
                        PrinterHandler::file_handler(songs, detail, output, add, level);
                    } else {
                        error_handler();
                    }
                }
                Some(MarkdownSubCommands::Id {
                    ids,
                    output,
                    detail,
                    add,
                    level,
                }) => {
                    let songs = ids
                        .iter()
                        .flat_map(|id| DXProberClient::search_songs_by_id(*id))
                        .collect();
                    PrinterHandler::file_handler(songs, detail, output, add, level);
                }
            }
        }
    }
}

/// 定义需要处理的字段和对应的字符串表示
fn error_handler() {
    error!("参数错误,请使用 --help 或者 -h 查看详情");
    exit(exitcode::USAGE)
}
