extern crate clap;

use std::process::exit;

use clap::Parser;
use log::error;

use maimai_search_lib::clients::song_data;
use maimai_search_lib::clients::user_data::get_b50_data;
use maimai_search_lib::config::command::{MaimaiSearchArgs, MarkdownSubCommands, SubCommands};
use maimai_search_lib::config::consts::PROFILE;
use maimai_search_lib::config::profiles::Profile;
use maimai_search_lib::image::maimai_best_50::{BestList, DrawBest};
use maimai_search_lib::service::resource;
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
                let songs = song_data::search_songs_by_title(name.as_str(), args.count);
                PrinterHandler::console_handler(songs, args.detail, args.level);
            } else {
                error_handler();
            }
        }
        // ID 检索子命令
        Some(SubCommands::Id { ids, detail, level }) => {
            let songs = ids
                .iter()
                .flat_map(|id| song_data::search_songs_by_id(*id))
                .collect();
            PrinterHandler::console_handler(songs, detail, level);
        }
        // 更新数据库子命令
        Some(SubCommands::Update {}) => resource::update_songs_data(),
        // 更新资源文件子命令
        Some(SubCommands::Resource { force }) => resource::update_resource(force),
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
                        let songs = song_data::search_songs_by_title(name.as_str(), count);
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
                        .flat_map(|id| song_data::search_songs_by_id(*id))
                        .collect();
                    PrinterHandler::file_handler(songs, detail, output, add, level);
                }
            }
        }

        Some(SubCommands::B50 { username }) => {
            let username = match username {
                None => {
                    match PROFILE.remote_api.maimaidxprober.username.clone() {
                        Some(username) => username,
                        None => {
                            error!("未指定用户名,请在配置文件中指定用户名或者使用 --username 指定用户名");
                            exit(exitcode::USAGE)
                        }
                    }
                }
                Some(username) => username
            };
            let resp = match get_b50_data(username.as_str()) {
                Ok(resp) => resp,
                Err(e) => {
                    error!("获取数据失败: {}", e);
                    exit(exitcode::NOHOST);
                }
            };
            let dx_charts = resp.charts.dx;
            let mut dx_best_list = BestList::new(15);
            for chart in dx_charts {
                dx_best_list.push(chart)
            }
            let sd_charts = resp.charts.sd;
            let mut sd_best_list = BestList::new(35);
            for chart in sd_charts {
                sd_best_list.push(chart)
            }
            let mut draw_best = DrawBest::new(sd_best_list, dx_best_list, &*resp.nickname);
            match draw_best.draw() {
                Ok(_) => {}
                Err(e) => {
                    error!("绘制失败: {}", e);
                    exit(exitcode::SOFTWARE);
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
