extern crate clap;

use std::process::exit;

use clap::Parser;
use log::{error, info};

use crate::command::{MaimaiSearchArgs, MarkdownSubCommands, SubCommands};
use maimai_search_lib::clients::song_data;
use maimai_search_lib::clients::user_data::get_b50_data;
use maimai_search_lib::config::profiles::Profile;
use maimai_search_lib::service::maimai_best_50::{BestList, DrawBest};
use maimai_search_lib::service::printer::PrinterHandler;
use maimai_search_lib::service::resource;
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
            Profile::open_config()
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
                    match Profile::get_username() {
                        Some(username) => username,
                        None => {
                            error!("未指定用户名,请在配置文件中指定用户名或者使用 --username 指定用户名");
                            exit(exitcode::USAGE)
                        }
                    }
                }
                Some(username) => username,
            };
            let resp = match get_b50_data(username.as_str()) {
                Ok(resp) => resp,
                Err(e) => {
                    error!("获取数据失败: {}", e);
                    exit(exitcode::NOHOST);
                }
            };
            info!("用户[{}]的成绩信息已载入,开始绘制", &resp.nickname);
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

mod simple_log {
    use colored::Colorize;
    use log::{Level, Metadata, Record};
    use log::{LevelFilter, SetLoggerError};

    static LOGGER: SimpleLogger = SimpleLogger;

    struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            return metadata.level() <= Level::Info && metadata.target().starts_with("maimai");
        }
        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let args = record.args();
                match record.level() {
                    Level::Error => {
                        eprintln!("{}{} {}", "error".red().bold(), ":".bold(), args);
                    }
                    Level::Warn => {
                        println!("{}{} {}", "warning".yellow().bold(), ":".bold(), args);
                    }
                    Level::Info => {
                        println!("{}{} {}", "info".green().bold(), ":".bold(), args);
                    }
                    _ => {}
                }
            }
        }
        fn flush(&self) {}
    }

    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
    }
}

mod command {
    use clap::{Parser, Subcommand};
    use maimai_search_lib::clients::user_data::entity::LevelLabel;

    /// GitHub Repository : [https://github.com/Anselyuki/maimai-search-rs]
    #[derive(Parser)]
    #[command(name = "maimai-search", bin_name = "maimai-search")]
    #[command(author, about, version, next_line_help = false)]
    pub struct MaimaiSearchArgs {
        /// 检索信息,如果打不出片假名没有关系,可以试试只把中文打进去(君の日本语本当上手)
        pub name: Option<String>,
        /// 模糊查询的匹配数量(由于实现比较简陋,往后的匹配结果可能会过于离谱)
        #[arg(short, long, default_value = "5")]
        pub count: usize,
        /// 开启详情查询
        #[arg(short, long)]
        pub detail: bool,
        /// 谱面等级
        #[arg(short, long, value_enum)]
        pub level: Option<LevelLabel>,
        // 子命令枚举
        #[command(subcommand)]
        pub command: Option<SubCommands>,
    }

    #[derive(Subcommand)]
    pub enum SubCommands {
        ///  使用 ID 进行检索，如：maimai-search id 11571 11524
        Id {
            /// 检索 ID ,支持多个 ID 检索
            ids: Vec<usize>,
            /// 谱面等级
            #[arg(short, long, value_enum)]
            level: Option<LevelLabel>,
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
            #[arg(short, long, default_value = "5")]
            count: usize,
            /// 开启详情查询
            #[arg(short, long)]
            detail: bool,
            /// 指定 markdown 输出的文件名称(路径使用当前程序执行的路径)
            #[arg(short, long, value_name = "MARKDOWN_FILE_NAME")]
            output: Option<String>,
            /// 以追加方式添加到 markdown 文件中
            #[arg(short, long)]
            add: Option<String>,
            /// 谱面等级
            #[arg(short, long, value_enum)]
            level: Option<LevelLabel>,
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
        /// 生成 B50 图片
        B50 {
            /// 用户名,可选参数,如果不填写则使用配置文件中的用户名
            username: Option<String>,
        },
    }

    /// 使用 markdown 格式输出
    #[derive(Subcommand)]
    pub enum MarkdownSubCommands {
        Id {
            /// 检索 ID ,支持多个 ID 检索
            ids: Vec<usize>,
            /// 指定 markdown 输出的文件名称(路径使用当前程序执行的路径)
            #[arg(short, long, value_name = "MARKDOWN_FILE_NAME")]
            output: Option<String>,
            /// 开启详情查询
            #[arg(short, long)]
            detail: bool,
            /// 以追加方式添加到 markdown 文件中
            #[arg(short, long)]
            add: Option<String>,
            /// 谱面等级
            #[arg(short, long, value_enum)]
            level: Option<LevelLabel>,
        },
    }
}
