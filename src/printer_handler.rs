use std::cmp::max;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::string::ToString;
use std::vec::Vec;

use colored::Colorize;
use prettytable::{Cell, format, row, Row, Table};

use crate::{DIFFICULT_NAME, LAUNCH_PATH, MARKDOWN_TABLE_STYLE};
use crate::client::Song;

pub struct PrinterHandler {}

struct SongTable {
    pub info: String,
    pub table: Table,
    pub heading_level: HeadingLevel,
}

/// 对应 markdown 内的标题等级
enum HeadingLevel {
    Two,
    Three,
}

struct TableUtil {}

struct ConsolePrinter {}

struct MarkdownPrinter {}

impl TableUtil {
    /// 批量获取歌曲的基本信息列表
    fn get_songs(songs: Vec<Song>, markdown: bool) -> Vec<SongTable> {
        let mut table = Table::new();
        let mut header = row!["ID","乐曲标题","分区","BPM"];

        // 检查这一批歌曲中最大的谱面数量
        let chart_count = songs.iter()
            .map(|song| { max(song.ds.len(), song.level.len()) })
            .max()
            .unwrap_or(0);

        for difficult in &DIFFICULT_NAME[..chart_count] {
            header.add_cell(difficult.clone());
        }
        table.set_titles(header);

        // 构建表格行
        for song in &songs {
            let title = match markdown {
                true => { format!("`{}`{}", song.song_type, song.title) }
                false => { format!("[{}]{}", song.song_type, song.title) }
            };
            let mut table_data = row![song.id,title,song.basic_info.genre,song.basic_info.bpm];
            for (ds, level) in song.ds.iter().zip(song.level.iter()) {
                let level_str = match Self::get_level_str(ds, level) {
                    Some(value) => value,
                    None => continue,
                };
                table_data.add_cell(Cell::new(level_str.as_str()));
            }
            table.add_row(table_data);
        }
        return vec![SongTable { info: "歌曲列表".to_string(), table, heading_level: HeadingLevel::Two }];
    }

    /// 批量输出歌曲的详细信息
    ///
    /// 图片信息经拼接得到例子如下:
    ///
    /// `![PANDORA PARADOXXX](https://www.diving-fish.com/covers/00834.png)`
    fn get_songs_detail(songs: Vec<Song>, markdown: bool) -> Vec<SongTable> {
        let mut table_vec = Vec::new();
        let mut song_map: HashMap<String, Vec<Song>> = HashMap::new();

        // 将 DX 谱和标准谱合在一起
        for song in songs {
            let mut song_vec = song_map.get(&song.clone().title).unwrap_or(&vec![]).to_vec();
            song_vec.push(song.clone());
            song_map.insert(song.title, song_vec);
        }

        for (title, songs) in song_map {
            let info = format!("乐曲情报:`{}`", title);
            if markdown {
                let mut table = Table::new();
                table.set_titles(row!["谱面图片","ID","乐曲标题","类型","分区","BPM","演唱/作曲"]);

                // 获取 md 内嵌的 图片字段
                for song in songs.clone() {
                    let pic_url = format!("![{}](https://www.diving-fish.com/covers/{:0>5}.png)", &song.title, &song.id);
                    // 其他直接可以用的列
                    let mut row = row![format!("{:5}", song.id), song.title,song.song_type,song.basic_info.genre,song.basic_info.bpm,song.basic_info.artist];
                    row.insert_cell(0, Cell::new(&*pic_url));
                    table.add_row(row);
                }
                // 乐曲情报构造完毕
                table_vec.push(SongTable { info, table, heading_level: HeadingLevel::Two });
            } else {
                let mut table = Table::new();
                table.set_titles(row!["ID","乐曲标题","类型","分区","BPM","演唱/作曲"]);
                for song in songs.clone() {
                    let row = row![format!("{:5}", song.id), song.title,song.song_type,song.basic_info.genre,song.basic_info.bpm,song.basic_info.artist];
                    table.add_row(row);
                }
                // 乐曲情报构造完毕
                table_vec.push(SongTable { info, table, heading_level: HeadingLevel::Two });
            }

            // 插入谱面信息表
            for song in songs {
                let chart_table = Self::get_chart_table(song);
                table_vec.push(chart_table);
            }
        }
        return table_vec;
    }

    /// 获取等级字符串
    fn get_level_str(ds: &f32, level: &String) -> Option<String> {
        // 将浮点数转换为字符串
        let num_str = ds.to_string();
        // 切分字符串，获取小数部分
        let decimal_part: String = num_str.chars()
            .skip_while(|&c| c != '.')
            .collect();
        if decimal_part.is_empty() {
            return Some(format!("{}({})", level, ".0"));
        }
        Some(format!("{}({})", level, decimal_part))
    }

    /// 每张谱面的详细信息
    fn get_chart_table(song: Song) -> SongTable {
        let mut table = Table::new();
        let mut title = row!["难度", "定数", "COMBO", "TAP", "HOLD", "SLIDE", "BREAK", "谱面作者"];
        let info =
            match song.song_type.as_str() {
                "DX" => {
                    title.insert_cell(6, Cell::new("TOUCH"));
                    "DX谱面情报".to_string()
                }
                "SD" => { "标准谱面情报".to_string() }
                _ => { panic!("数据库难度列错误") }
            };
        table.set_titles(title);
        // 构建谱面信息
        for ((chart, ds), difficult) in song.charts.iter().zip(song.ds.iter()).zip(DIFFICULT_NAME.iter()) {
            let mut table_data = Row::empty();
            table_data.add_cell(difficult.clone());
            table_data.add_cell(Cell::new(&*ds.to_string()));

            // 添加谱面的详细信息
            let notes = &chart.notes;
            table_data.add_cell(Cell::new(&*format!("{}", notes.iter().sum::<u32>())));
            for note in notes {
                table_data.add_cell(Cell::new(&*format!("{}", note)));
            }
            // 添加谱面作者
            table_data.add_cell(Cell::new(&chart.charter));
            table.add_row(table_data);
        };
        return SongTable { info, table, heading_level: HeadingLevel::Three };
    }
}

impl PrinterHandler {
    /// 输出信息
    pub(crate) fn new(songs: Vec<Song>, detail: bool, markdown: bool, output: Option<String>) {
        let table_vec = match detail {
            true => { TableUtil::get_songs_detail(songs, markdown) }
            false => { TableUtil::get_songs(songs, markdown) }
        };
        if markdown {
            match output {
                // 写入 md 文件
                Some(filename) => MarkdownPrinter::write_file(filename, table_vec),
                // 输出 md 格式的表格在命令行
                None => ConsolePrinter::print_std(table_vec, markdown)
            }
        } else {
            ConsolePrinter::print_std(table_vec, false);
        }
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
                    HeadingLevel::Three => "###"
                };
                println!("{} {}\n", heading, song_table.info);
                table.set_format(*MARKDOWN_TABLE_STYLE);
            } else {
                table.set_format(*format::consts::FORMAT_BOX_CHARS);
            }
            table.printstd();
        }
    }
}

impl MarkdownPrinter {
    fn write_file(filename: String, song_tables: Vec<SongTable>) {
        let path = Self::add_md_extension(filename);
        let version = env!("CARGO_PKG_VERSION");
        let name = env!("CARGO_PKG_NAME");
        let repo = env!("CARGO_PKG_REPOSITORY");
        let info_str = format!("> create by maimai-search {}\n>\n> GitHub Repository : [{}]({})\n", version, name, repo);
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
                HeadingLevel::Three => "###"
            };
            table.set_format(*MARKDOWN_TABLE_STYLE);
            let table_str = table.to_string();
            writeln!(file, "{} {}\n", heading, song_table.info).unwrap();
            writeln!(file, "{}", table_str).unwrap();
        }
        println!("{}: 文件写入结束，文件路径:[{}]", "info".green().bold(), path.display());
        if let Err(error) = open::that(path) {
            eprintln!("{}: 无法打开文件: {:?}", "error".red().bold(), error);
        }
    }

    /// 验证文件名合法性
    ///
    /// - 如果输入是 md 文件，则原封不动的返回路径
    /// - 如果输入文件没有拓展名，则为其添加
    /// - 如果输入文件携带非 md 的扩展名，则报错
    fn add_md_extension(filename: String) -> PathBuf {
        let file_path = &LAUNCH_PATH.join(filename);
        let path = file_path.as_path();
        match path.extension() {
            Some(ext) => {
                if ext == "md" {
                    print!("{}: 将创建文件{:?}...", "info".green().bold(), path.file_name().unwrap());
                    return path.to_owned();
                } else {
                    eprintln!("{}: 文件后缀不是\".md\",获取到\".{}\",可以选择不指定后缀名,或指定\".md\"后缀名", "error".red().bold(), ext.to_str().unwrap());
                    exit(exitcode::USAGE);
                }
            }
            None => {
                let mut new_path = PathBuf::from(path);
                new_path.set_extension("md");
                println!("{}: 将创建文件{:?}", "info".green().bold(), new_path.file_name().unwrap());
                return new_path;
            }
        }
    }
}
