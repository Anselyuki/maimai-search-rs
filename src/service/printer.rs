use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::exit;
use std::string::ToString;
use std::vec::Vec;

use log::{error, info, warn};
use prettytable::format::consts::FORMAT_BOX_CHARS;

use crate::clients::song_data::entity::Song;
use crate::config::command::ChartLevel;
use crate::config::consts::{MARKDOWN_TABLE_STYLE, PROFILE};
use crate::service::table::{SongTable, TableService};
use crate::utils::file::FileUtils;

pub struct PrinterHandler;

struct ConsolePrinter;

struct FilePrinter;

impl PrinterHandler {
    /// Console 输出处理器
    pub fn console_handler(songs: Vec<Song>, detail: bool, level: Option<ChartLevel>) {
        let table_vec = match detail {
            true => TableService::get_songs_detail(
                songs,
                PROFILE.markdown.picture.console_picture,
                None,
            ),
            false => TableService::get_songs(
                songs,
                PROFILE.markdown.picture.console_picture,
                None,
                level,
            ),
        };
        ConsolePrinter::print_std(table_vec, false);
    }

    /// Markdown 格式处理器
    pub fn file_handler(
        songs: Vec<Song>,
        detail: bool,
        output: Option<String>,
        add: Option<String>,
        level: Option<ChartLevel>,
    ) {
        // 输出到文件的都添加图片列,输出到 Console 的根据配置文件决定
        let pic_colum = match (
            add.clone(),
            output.clone(),
            PROFILE.markdown.picture.console_picture,
        ) {
            (None, None, console_picture) => console_picture,
            _ => true,
        };

        let table_vec = match detail {
            true => TableService::get_songs_detail(songs, pic_colum, output.clone()),
            false => TableService::get_songs(songs, pic_colum, output.clone(), level),
        };

        // 输出到文件
        if let Some(filename) = output {
            FilePrinter::write_markdown_file(filename, table_vec);
            exit(exitcode::OK)
        };

        // 尾部追加模式
        if let Some(filename) = add {
            FilePrinter::addition_file(filename, table_vec);
            exit(exitcode::OK)
        }

        // 文件相关指令都没有开启,输出 markdown 格式在命令行
        ConsolePrinter::print_std(table_vec, true);
    }
}

impl ConsolePrinter {
    /// 输出表格的详细信息
    fn print_std(song_tables: Vec<SongTable>, markdown: bool) {
        if song_tables.is_empty() {
            warn!("找不到对应的歌曲!");
            exit(exitcode::DATAERR)
        }
        for song_table in song_tables {
            let mut table = song_table.table;
            if markdown {
                println!("\n{} {}\n", song_table.head.to_string(), song_table.info);
                table.set_format(*MARKDOWN_TABLE_STYLE);
            } else {
                println!("[{}]", song_table.info);
                table.set_format(*FORMAT_BOX_CHARS);
            }
            table.printstd();
        }
    }
}

/// # 文件输出器
///
/// 用来处理指定了相应的文件输出功能
impl FilePrinter {
    /// # 新建文件(覆盖式)
    ///
    /// 这个模式下会覆盖之前可能存在的文件,用新的内容覆盖它
    fn write_markdown_file(filename: String, song_tables: Vec<SongTable>) {
        let path = FileUtils::add_md_extension(filename);
        // 创建文件,文件不存在会创建文件
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(e) => panic!("Error creating file: {:?}", e),
        };

        let info_str = format!(
            "> create by maimai-search {}\n>\n> GitHub Repository : [{}]({})\n",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_REPOSITORY")
        );
        writeln!(file, "{}", info_str).unwrap();
        Self::write_file(song_tables, &mut file, true);

        info!("文件成功写入:[{}]", path.display());
        if let Err(error) = open::that(path) {
            error!("无法打开文件: {:?}", error);
        }
    }

    /// # 在文件尾部追加内容
    ///
    /// 在指定的 markdown 文件尾部指定追加表格
    ///
    /// - 不会再输出表格标题
    /// - 不会再输出版权信息
    pub fn addition_file(filename: String, song_tables: Vec<SongTable>) {
        let path = FileUtils::add_md_extension(filename);
        // 创建文件,文件不存在会创建文件
        let mut file = match OpenOptions::new().append(true).open(&path) {
            Ok(file) => file,
            Err(e) => panic!("Error Open file: {:?}", e),
        };
        Self::write_file(song_tables, &mut file, false);
        info!("文件成功写入:[{}]", &path.display());
        if let Err(error) = open::that(&path) {
            error!("无法打开文件: {:?}", error);
        }
    }

    /// 向文件内写入内容,写入模式由传入的文件决定
    fn write_file(song_tables: Vec<SongTable>, file: &mut File, has_title: bool) {
        for song_table in song_tables {
            let mut table = song_table.table;
            table.set_format(*MARKDOWN_TABLE_STYLE);
            let table_str = table.to_string();
            if has_title {
                writeln!(
                    file,
                    "{} {}\n",
                    song_table.head.to_string(),
                    song_table.info
                )
                .unwrap();
            }
            writeln!(file, "{}", table_str).unwrap();
        }
    }
}
