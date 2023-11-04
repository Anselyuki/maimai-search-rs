use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::string::ToString;
use std::vec::Vec;

use log::{error, info, warn};
use prettytable::format::consts::FORMAT_BOX_CHARS;

use crate::config::consts::{MARKDOWN_TABLE_STYLE, PROFILE};
use crate::db::entity::Song;
use crate::service::table::{HeadingLevel, SongTable, TableService};
use crate::utils::file::FileUtils;

pub struct PrinterHandler {}

struct ConsolePrinter {}

struct MarkdownPrinter {}

impl PrinterHandler {
    /// 输出信息
    pub fn new(
        songs: Vec<Song>,
        detail: bool,
        markdown: bool,
        output: Option<String>,
        add: Option<String>,
    ) {
        // 输出到文件的都添加图片列,输出到 Console 的根据配置文件决定
        let pic_colum = match (output.clone(), PROFILE.markdown.picture.console_picture) {
            (None, console_picture) => console_picture,
            (Some(_), _) => true,
        };

        let table_vec = match detail {
            true => TableService::get_songs_detail(songs, pic_colum, output.clone()),
            false => TableService::get_songs(songs, pic_colum, output.clone()),
        };
        // 是否输出到文件
        if let Some(filename) = output {
            match markdown {
                // 写入 md 文件
                true => MarkdownPrinter::write_file(filename, table_vec),
                // 输出 md 格式的表格在命令行,提示
                false => Self::markdown_rollback(markdown, table_vec),
            }
            exit(exitcode::OK)
        };

        if let Some(filename) = add {
            match markdown {
                // 写入 md 文件
                true => MarkdownPrinter::write_file(filename, table_vec),
                // 输出 md 格式的表格在命令行,提示
                false => Self::markdown_rollback(markdown, table_vec),
            }
            exit(exitcode::OK)
        }
        ConsolePrinter::print_std(table_vec, markdown);
    }

    fn markdown_rollback(markdown: bool, table_vec: Vec<SongTable>) {
        warn!("未指定 markdown 输出! 使用 --markdown(-md) 开启 markdown 输出");
        ConsolePrinter::print_std(table_vec, markdown)
    }
}

impl ConsolePrinter {
    /// 输出表格的详细信息
    fn print_std(song_tables: Vec<SongTable>, markdown: bool) {
        for song_table in song_tables {
            let mut table = song_table.table;
            if markdown {
                let heading = match song_table.heading_level {
                    HeadingLevel::Two => "##",
                    HeadingLevel::Three => "###",
                };
                println!("\n{} {}\n", heading, song_table.info);
                table.set_format(*MARKDOWN_TABLE_STYLE);
            } else {
                println!("[{}]", song_table.info);
                table.set_format(*FORMAT_BOX_CHARS);
            }
            table.printstd();
        }
    }
}

impl MarkdownPrinter {
    /// 新建文件(覆盖式)
    fn write_file(filename: String, song_tables: Vec<SongTable>) {
        let path = FileUtils::add_md_extension(filename);
        let version = env!("CARGO_PKG_VERSION");
        let name = env!("CARGO_PKG_NAME");
        let repo = env!("CARGO_PKG_REPOSITORY");
        let info_str = format!(
            "> create by maimai-search {}\n>\n> GitHub Repository : [{}]({})\n",
            version, name, repo
        );
        // 打开文件并写入yaml字符串
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(e) => panic!("Error creating file: {:?}", e),
        };

        writeln!(file, "{}", info_str).unwrap();
        for song_table in song_tables {
            let mut table = song_table.table;
            let heading = match song_table.heading_level {
                HeadingLevel::Two => "##",
                HeadingLevel::Three => "###",
            };
            table.set_format(*MARKDOWN_TABLE_STYLE);
            let table_str = table.to_string();
            writeln!(file, "{} {}\n", heading, song_table.info).unwrap();
            writeln!(file, "{}", table_str).unwrap();
        }
        info!("文件成功写入:[{}]", path.display());
        if let Err(error) = open::that(path) {
            error!("无法打开文件: {:?}", error);
        }
    }

    // fn addition_file() {
    //     //TODO 追加模式写入md数据
    // }
}
