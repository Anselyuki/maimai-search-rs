use prettytable::{Cell, format, row, Table};

use crate::entity::Song;

pub struct Printer {}

impl Printer {
    /// 打印歌曲列表
    pub fn print_detail(songs: Vec<Song>) {
        let mut table = Table::new();
        let mut header = row!["ID","乐曲标题","类型","分区","BPM"];
        let difficult_header = vec!["绿", "黄", "红", "紫", "白"];

        // 检查这一批歌曲中最大的谱面数量
        let chart_count = songs.iter()
            .map(|song| {
                std::cmp::max(song.ds.len(), song.level.len())
            })
            .max()
            .unwrap_or(0);


        for header_text in &difficult_header[..chart_count] {
            header.add_cell(Cell::new(&*header_text.to_string()));
        }
        table.set_titles(header);

        // 构建表格行
        for song in &songs {
            let title = format!("[{}]{}", song.song_type, song.title);
            let mut table_data = row![&song.id,title,&song.song_type,&song.basic_info.genre,&song.basic_info.bpm];
            for (ds, level) in song.ds.iter().zip(song.level.iter()) {
                // 将浮点数转换为字符串
                let num_str = ds.to_string();
                // 切分字符串，获取小数部分
                let decimal_part: String = num_str.chars()
                    .skip_while(|&c| c != '.')
                    .collect();
                if decimal_part.is_empty() {
                    table_data.add_cell(Cell::new(&*format!("{}({})", level, ".0")));
                    continue;
                }
                let level_str = format!("{}({})", level, decimal_part);
                table_data.add_cell(Cell::new(level_str.as_str()));
            }
            table.add_row(table_data);
        }
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.printstd();
    }
}