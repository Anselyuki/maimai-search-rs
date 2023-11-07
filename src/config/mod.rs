pub mod profiles;

pub mod consts {
    extern crate lazy_static;

    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use platform_dirs::AppDirs;
    use prettytable::color::{GREEN, MAGENTA, RED, WHITE, YELLOW};
    use prettytable::format::*;
    use prettytable::{Attr, Cell};

    use crate::config::profiles::Profile;

    lazy_static! {
        // 在 MacOS下遵守 XDG 规范,即创建的配置文件夹为 `~/.config/maimai-search`
        pub static ref CONFIG_PATH: PathBuf = AppDirs::new(Some("maimai-search"), true).unwrap().config_dir;
        pub static ref PROFILE: Profile = Profile::new();
        pub static ref DIFFICULT_NAME: Vec<Cell> = vec!["BASIC", "ADVANCED", "EXPERT", "MASTER", "Re:MASTER"].iter()
            .zip(&[GREEN, YELLOW, RED, MAGENTA, WHITE])
            .map(|(difficult, column_color)| Cell::new(difficult).with_style(Attr::ForegroundColor(*column_color)))
            .collect();
        pub static ref LAUNCH_PATH: PathBuf = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        pub static ref MARKDOWN_TABLE_STYLE: TableFormat = FormatBuilder::new()
            .column_separator('|').borders('|')
            .separators(&[LinePosition::Title], LineSeparator::new('-', '|', '|', '|'))
            .padding(1, 1).build();
    }
}

pub mod command {
    use clap::{Parser, Subcommand};

    /// GitHub Repository : [https://github.com/Anselyuki/maimai-search-rs]
    #[derive(Parser, Debug)]
    #[command(name = "maimai-search", bin_name = "maimai-search")]
    #[command(author, about, version, next_line_help = false)]
    pub struct MaimaiSearchArgs {
        // 子命令枚举
        #[command(subcommand)]
        pub command: Option<SubCommands>,
        /// 检索信息,如果打不出片假名没有关系,可以试试只把中文打进去(君の日本语本当上手)
        pub name: Option<String>,
        /// 模糊查询的匹配数量(由于实现比较简陋,往后的匹配结果可能会过于离谱)
        #[arg(short, long, default_value = "3")]
        pub count: usize,
        /// 开启详情查询
        #[arg(short, long)]
        pub detail: bool,
    }

    #[derive(Subcommand, Debug)]
    pub enum SubCommands {
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
            #[arg(short, long, value_name = "MARKDOWN_FILE_NAME")]
            output: Option<String>,
            /// 以追加方式添加到 markdown 文件中
            #[arg(short, long)]
            add: Option<String>,
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
        },
    }
}
