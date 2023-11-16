use std::cmp::max;
use std::collections::HashMap;
use std::fmt;
use std::fs::create_dir;
use std::path::Path;
use std::process::exit;

use log::{error, warn};
use prettytable::{Cell, row, Row, Table};

use crate::clients::song_data::entity::Song;
use crate::clients::user_data::entity::LevelLabel;
use crate::config::consts::{CONFIG_PATH, DIFFICULT_NAME, LAUNCH_PATH, PROFILE};
use crate::service::resource::update_resource;
use crate::utils::file::{copy_file, remove_extension};

/// 歌曲列表
///
/// 这个是用于最后输出的表格格式,具体意义看字段说明
pub struct SongTable {
    /// 表格信息
    pub info: String,
    /// 表格内容
    pub table: Table,
    /// 表格头等级
    pub head: MarkdownFormat,
}

/// # Markdown 枚举
///
/// 对应 markdown 内的语法枚举
///
/// 例如`MarkdownFormat::H1`表示了一级标题,后续如果有需要转换的可以在这里添加

/// 可以调用这个枚举上的 `to_string()` 方法获取对应的 Markdown 字串
#[warn(unused)]
pub enum MarkdownFormat {
    /// 一级标题
    H1,
    /// 二级标题
    H2,
    /// 三级标题
    H3,
}

impl fmt::Display for MarkdownFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MarkdownFormat::H1 => "#",
                MarkdownFormat::H2 => "##",
                MarkdownFormat::H3 => "###",
            }
        )
    }
}

pub struct TableService;

impl TableService {
    /// 批量获取歌曲的基本信息列表
    pub fn get_songs(
        songs: Vec<Song>,
        pic_colum: bool,
        output: &Option<String>,
        level: Option<LevelLabel>,
    ) -> Vec<SongTable> {
        let mut table = Table::new();
        let mut header = row!["ID", "乐曲标题", "分区", "BPM"];
        if pic_colum {
            header.insert_cell(0, Cell::new("谱面图片"))
        }

        // 检查这一批歌曲中最大的谱面数量
        let chart_count = songs
            .iter()
            .map(|song| max(song.ds.len(), song.level.len()))
            .max()
            .unwrap_or(0);

        if let Some(chart_level) = &level {
            let cell = &DIFFICULT_NAME[*chart_level as usize];
            header.add_cell(cell.clone());
        } else {
            for difficult in &DIFFICULT_NAME[..chart_count] {
                header.add_cell(difficult.clone());
            }
        }
        table.set_titles(header);

        // 构建表格行
        for song in &songs {
            let title = if pic_colum {
                format!("`{}`{}", song.song_type, song.title)
            } else {
                format!("[{}]{}", song.song_type, song.title)
            };

            let mut table_data = match pic_colum {
                true => {
                    let pic_url = Self::get_song_picture(&song, &output);
                    row![
                        pic_url,
                        song.id,
                        title,
                        song.basic_info.genre,
                        song.basic_info.bpm
                    ]
                }
                false => {
                    row![song.id, title, song.basic_info.genre, song.basic_info.bpm]
                }
            };

            // 指定难度的谱面
            if let Some(chart_level) = level{
                let index = chart_level as usize;
                table_data.add_cell(Cell::new(
                    Self::get_level_str(&song.ds[index], &song.level[index])
                        .unwrap()
                        .as_str(),
                ))
            } else {
                for (ds, level) in song.ds.iter().zip(song.level.iter()) {
                    let level_str = match Self::get_level_str(ds, level) {
                        Some(value) => value,
                        None => continue,
                    };
                    table_data.add_cell(Cell::new(level_str.as_str()));
                }
            }
            table.add_row(table_data);
        }
        return vec![SongTable {
            info: "歌曲列表".to_string(),
            table,
            head: MarkdownFormat::H2,
        }];
    }

    /// 批量输出歌曲的详细信息
    pub fn get_songs_detail(
        songs: Vec<Song>,
        pic_colum: bool,
        output: &Option<String>,
    ) -> Vec<SongTable> {
        let mut table_vec = Vec::new();
        let mut song_map: HashMap<String, Vec<Song>> = HashMap::new();

        // 将 DX 谱和标准谱合在一起
        for song in songs {
            let mut song_vec = song_map
                .get(&song.title)
                .unwrap_or(&vec![])
                .to_vec();
            song_vec.push(song.clone());
            song_map.insert(song.title, song_vec);
        }

        for (title, songs) in song_map {
            let mut table = Table::new();
            // 构建表头
            table.set_titles(if pic_colum {
                row![
                    "谱面图片",
                    "ID",
                    "乐曲标题",
                    "类型",
                    "分区",
                    "BPM",
                    "演唱/作曲"
                ]
            } else {
                row!["ID", "乐曲标题", "类型", "分区", "BPM", "演唱/作曲"]
            });

            // 表格内容
            for song in songs.iter() {
                let mut row = row![
                    format!("{:5}", song.id),
                    song.title,
                    song.song_type,
                    song.basic_info.genre,
                    song.basic_info.bpm,
                    song.basic_info.artist
                ];
                // 插入图片 URL
                if pic_colum {
                    let pic_url = Self::get_song_picture(&song, output);
                    row.insert_cell(0, Cell::new(&*pic_url));
                }
                table.add_row(row);
            }

            // 乐曲情报构造完毕
            table_vec.push(SongTable {
                table,
                info: format!("乐曲情报 : {}", title),
                head: MarkdownFormat::H2,
            });

            // 插入谱面信息表
            table_vec.extend(
                songs
                    .into_iter()
                    .map(|song| Self::get_chart_table(song))
                    .collect::<Vec<SongTable>>(),
            );
        }
        return table_vec;
    }

    /// 获得图片URL
    ///
    /// 如果开启了本地化图片并且输出有值则会执行文件操作,图片信息经拼接得到例子如下:
    ///
    /// `![PANDORA PARADOXXX](https://www.diving-fish.com/covers/00834.png)`
    fn get_song_picture(song: &Song, output: &Option<String>) -> String {
        let config = &PROFILE.markdown.picture;
        if !config.local.enable || output.is_none() {
            return format!(
                "![{}]({}{:0>5}.png)",
                &song.title, config.remote.prefix_url, &song.id
            );
        }

        // 如果开启了本地化图片并且输出有值
        let output_name = remove_extension(output.clone().unwrap());
        // 是否开启绝对路径
        let mut absolute = &PROFILE.markdown.picture.local.absolute;
        let res_dir = match &PROFILE.markdown.picture.local.path {
            None => LAUNCH_PATH.join(&output_name),
            Some(path) => {
                if !absolute {
                    warn!("开启自定义资源目录时不支持相对路径引用");
                    absolute = &true;
                }
                Path::new(path).to_path_buf()
            }
        };

        if !res_dir.exists() {
            if let Err(error) = create_dir(&res_dir) {
                error!("创建图片文件夹失败\n[Cause]:{:?}", error);
                exit(exitcode::IOERR)
            }
        };

        let filename = format!("{:0>5}.png", &song.id);
        let source_path = CONFIG_PATH.join("resource/mai/cover").join(&filename);
        // 资源文件夹不存在,执行一次资源更新
        if !source_path.exists() {
            warn!("资源文件不存在,执行资源文件更新");
            update_resource(false);
        }

        if let Err(error) = copy_file(source_path, res_dir.join(&filename)) {
            error!("拷贝资源文件失败!使用远程地址\n[Cause]:{:?}", error);
            return format!(
                "![{}]({}{:0>5}.png)",
                &song.title, config.remote.prefix_url, &song.id
            );
        }

        format!(
            "![{}]({}/{})",
            &song.title,
            if *absolute {
                res_dir.display().to_string()
            } else {
                output_name
            },
            filename
        )
    }

    /// 获取等级字符串
    fn get_level_str(ds: &f32, level: &String) -> Option<String> {
        // 将浮点数转换为字符串
        let num_str = ds.to_string();
        // 切分字符串，获取小数部分
        let decimal_part: String = num_str.chars().skip_while(|&c| c != '.').collect();
        if decimal_part.is_empty() {
            return Some(format!("{}({})", level, ".0"));
        }
        Some(format!("{}({})", level, decimal_part))
    }

    /// 每张谱面的详细信息
    fn get_chart_table(song: Song) -> SongTable {
        let mut table = Table::new();
        let mut title = row![
            "难度",
            "定数",
            "COMBO",
            "TAP",
            "HOLD",
            "SLIDE",
            "BREAK",
            "谱面作者"
        ];
        let info = match song.song_type.as_str() {
            "DX" => {
                title.insert_cell(6, Cell::new("TOUCH"));
                "DX谱面情报".to_string()
            }
            "SD" => "标准谱面情报".to_string(),
            _ => {
                error!("数据库难度列错误");
                exit(exitcode::DATAERR)
            }
        };
        table.set_titles(title);
        // 构建谱面信息
        for ((chart, ds), difficult) in song
            .charts
            .iter()
            .zip(song.ds.iter())
            .zip(DIFFICULT_NAME.iter())
        {
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
        }
        return SongTable {
            info,
            table,
            head: MarkdownFormat::H3,
        };
    }
}
