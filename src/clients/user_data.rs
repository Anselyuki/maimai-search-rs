use std::error::Error;
use std::process::exit;

use log::error;
use reqwest::blocking;
use serde_json::json;

use crate::clients::user_data::entity::B50Response;
use crate::config::consts::PROFILE;

/// 从远程服务器拿指定用户的 b50 数据
pub fn get_b50_data(username: &str) -> Result<B50Response, Box<dyn Error>> {
    let config = &PROFILE.remote_api.maimaidxprober;
    let payload = json!(
        {
            "username":username,
            "b50":true
        }
    );
    let request = blocking::Client::new()
        .post(&config.data_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload.to_string());
    let response = request.send()?;
    let status = response.status();
    Ok(match status.as_u16() {
        200 => {
            let resp_text: B50Response = response.json().unwrap();
            resp_text
        }
        400 => {
            error!("未找到此玩家，请确保此玩家的用户名和查分器中的用户名相同");
            exit(exitcode::NOUSER);
        }
        403 => {
            error!("该用户禁止了其他人获取数据");
            exit(exitcode::UNAVAILABLE);
        }
        _ => {
            error!("[{}] <-- http 请求错误", status);
            exit(exitcode::NOHOST);
        }
    })
}

pub mod entity {
    use std::cmp::Ordering;
    use std::fmt::Display;

    use image::Rgba;
    use serde::{Deserialize, Serialize};

    /// 查分器返回的数据
    #[derive(Serialize, Deserialize)]
    pub struct B50Response {
        /// 查分器用户名
        pub username: String,
        /// 谱面列表
        pub charts: Charts,
        /// 用户名( Maimai 机台上显示的)
        pub nickname: String,
        /// 底分
        pub rating: i32,
        /// 用户段位(查分器拿不到,所以是在查分器网站上设置几段就几段)
        pub additional_rating: i32,
        /// 又一个不知道干啥的,先放着
        pub plate: String,
        /// 又又一个不知道干啥的,先放着
        pub user_general_data: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Charts {
        pub dx: Vec<ChartInfoResponse>,
        pub sd: Vec<ChartInfoResponse>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ChartInfoResponse {
        /// 达成率
        pub achievements: f32,
        /// 谱面定数
        pub ds: f32,
        /// DX 分数
        #[serde(rename = "dxScore")]
        pub dx_score: i32,
        /// FULL COMBO
        pub fc: String,
        /// FULL SYNC
        pub fs: String,
        /// 等级
        pub level: String,
        /// 难度标签
        pub level_label: LevelLabel,
        /// 难度分
        pub ra: i32,
        /// 等级
        pub rate: ChartRate,
        /// 这里的 ID 跟 db 内的 ID 相关联的
        pub song_id: i32,
        /// 歌曲标题
        pub title: String,
        /// 歌曲类型
        #[serde(rename = "type")]
        pub song_type: String,
    }

    #[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone)]
    pub enum LevelLabel {
        Basic = 0,
        Advanced = 1,
        Expert = 2,
        Master = 3,
        #[serde(rename = "Re:MASTER")]
        ReMaster = 4,
    }

    impl Display for LevelLabel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let level_str = match self {
                LevelLabel::Basic => "BASIC",
                LevelLabel::Advanced => "ADVANCED",
                LevelLabel::Expert => "EXPERT",
                LevelLabel::Master => "MASTER",
                LevelLabel::ReMaster => "Re:MASTER",
            };
            write!(f, "{}", level_str)
        }
    }

    impl LevelLabel {
        /// 获取难度等级对应的颜色
        pub fn label_color(&self) -> Rgba<u8> {
            match self {
                LevelLabel::Basic => Rgba([69, 193, 36, 255]),
                LevelLabel::Advanced => Rgba([255, 186, 1, 255]),
                LevelLabel::Expert => Rgba([255, 90, 102, 255]),
                LevelLabel::Master => Rgba([134, 49, 200, 255]),
                LevelLabel::ReMaster => Rgba([217, 197, 233, 255]),
            }
        }
    }

    #[derive(PartialEq, PartialOrd, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ChartRate {
        D,
        C,
        B,
        BB,
        BBB,
        A,
        AA,
        AAA,
        S,
        SP,
        SS,
        SSP,
        SSS,
        SSSP,
    }

    impl Display for ChartRate {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let rate_str = match self {
                ChartRate::D => "D",
                ChartRate::C => "C",
                ChartRate::B => "B",
                ChartRate::BB => "BB",
                ChartRate::BBB => "BBB",
                ChartRate::A => "A",
                ChartRate::AA => "AA",
                ChartRate::AAA => "AAA",
                ChartRate::S => "S",
                ChartRate::SP => "S+",
                ChartRate::SS => "SS",
                ChartRate::SSP => "SS+",
                ChartRate::SSS => "SSS",
                ChartRate::SSSP => "SSS+",
            };
            write!(f, "{}", &rate_str)
        }
    }

    impl ChartRate {
        pub fn get_file_name(&self) -> String {
            format!(
                "UI_GAM_Rank_{}.png",
                match self {
                    ChartRate::D => "D",
                    ChartRate::C => "C",
                    ChartRate::B => "B",
                    ChartRate::BB => "BB",
                    ChartRate::BBB => "BBB",
                    ChartRate::A => "A",
                    ChartRate::AA => "AA",
                    ChartRate::AAA => "AAA",
                    ChartRate::S => "S",
                    ChartRate::SP => "Sp",
                    ChartRate::SS => "SS",
                    ChartRate::SSP => "SSp",
                    ChartRate::SSS => "SSS",
                    ChartRate::SSSP => "SSSp",
                }
            )
        }
    }

    impl PartialEq for ChartInfoResponse {
        fn eq(&self, other: &Self) -> bool {
            self.ra == other.ra
        }
    }

    impl PartialOrd for ChartInfoResponse {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Eq for ChartInfoResponse {}

    impl Ord for ChartInfoResponse {
        fn cmp(&self, other: &Self) -> Ordering {
            self.ra
                .cmp(&other.ra)
                .then_with(|| self.level_label.cmp(&other.level_label))
                .then_with(|| {
                    self.achievements
                        .partial_cmp(&other.achievements)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| self.title.cmp(&other.title))
        }
    }

    impl Display for ChartInfoResponse {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "[{} {}][{}] {}\n",
                self.level_label, self.level, self.song_type, self.title,
            )?;
            write!(
                f,
                "[{}][{}] Achievements: {:.4}%, DS: {:.1}, RA: {}, Rate: {} ",
                self.fc, self.fs, self.achievements, self.ds, self.ra, self.rate,
            )
        }
    }
}
