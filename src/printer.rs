use std::cmp::max;

use prettytable::{Cell, format, row, Row, Table};

use crate::DIFFICULT_NAME;
use crate::entity::Song;

pub struct Printer {}

impl Printer {
    /// 输出歌曲的基本信息列表
    pub fn print_songs_info(songs: Vec<Song>) {
        let mut table = Table::new();
        let mut header = row!["ID","乐曲标题","分区","BPM"];

        // 检查这一批歌曲中最大的谱面数量
        let chart_count = songs.iter()
            .map(|song| { max(song.ds.len(), song.level.len()) })
            .max()
            .unwrap_or(0);


        for difficult in &DIFFICULT_NAME[..chart_count] {
            header.add_cell(Cell::new(&*difficult));
        }
        table.set_titles(header);

        // 构建表格行
        for song in &songs {
            let title = format!("[{}]{}", song.song_type, song.title);
            let mut table_data = row![song.id,title,song.basic_info.genre,song.basic_info.bpm];
            Self::set_ds(song, &mut table_data);
            table.add_row(table_data);
        }
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.printstd();
    }

    fn set_ds(song: &Song, table_data: &mut Row) {
        for (ds, level) in song.ds.iter().zip(song.level.iter()) {
            let level_str = match Self::get_level_str(ds, level) {
                Some(value) => value,
                None => continue,
            };
            table_data.add_cell(Cell::new(level_str.as_str()));
        }
    }

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

    /// 输出单首歌曲的详细信息
    pub fn print_songs_detail(song: Song) {
        let mut table = Table::new();
        let chart_title = row!["难度", "等级", "COMBO", "TAP", "HOLD", "SLIDE", "BREAK", "谱面作者"];

        // 构建基本信息
        println!("乐曲情报");

        table.set_titles(row!["ID","乐曲标题","类型","分区","BPM","演唱/作曲"]);
        let basic_info = song.basic_info;
        table.add_row(row![format!("{:5}", song.id), song.title,song.song_type,basic_info.genre,basic_info.bpm,basic_info.artist]);
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.printstd();

        // 每张谱面的详细信息
        table = Table::new();
        println!("standard 谱面情报");
        // 构建谱面信息
        // 谱面数量
        table.set_titles(chart_title);
        for ((chart, level), difficult) in song.charts.iter().zip(song.level.iter()).zip(DIFFICULT_NAME.iter()) {
            let mut table_data = row![difficult,level];
            let notes = &chart.notes;
            table_data.add_cell(Cell::new(&*format!("{}", notes.iter().sum::<u32>())));
            for note in notes {
                table_data.add_cell(Cell::new(&*format!("{}", note)));
            }
            table_data.add_cell(Cell::new(&chart.charter));
            table.add_row(table_data);
        };

        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.printstd();
    }
}