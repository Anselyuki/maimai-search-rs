use crate::clients::user_data::entity::ChartInfoResponse;
use std::cmp;
use std::fmt::Display;

/// # 自排序 Best 列表
///
/// `ChartInfo` 实现了基于歌曲 RATING 的排序比较规则,根据这个规则实现了一个插入时排序的列表用于安放
///
/// - `BestList::new(15)` 可以创建一个大小为 15 的列表,用来装载 B15
/// - `BestList::new(35)` 可以创建一个大小为 35 的列表,用来装载 B35
///
/// > 这个结构体不知道要不要留,先按照 [mai-bot](https://github.com/Diving-Fish/mai-bot) 的规则来
///
/// 毕竟*最大头的还是PIL库的调用*
#[derive(Debug)]
pub struct BestList {
    data: Vec<ChartInfoResponse>,
    size: usize,
}

impl BestList {
    pub fn new(size: usize) -> Self {
        Self { data: vec![], size }
    }

    pub fn push(&mut self, elem: ChartInfoResponse) {
        if self.data.len() >= self.size && elem < *self.data.last().unwrap() {
            return;
        }
        self.data.push(elem);
        self.data.sort();
        self.data.reverse();
        while self.data.len() > self.size {
            self.data.pop();
        }
    }

    pub fn pop(&mut self) -> Option<ChartInfoResponse> {
        self.data.pop()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl std::fmt::Display for BestList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data_str = self
            .data
            .iter()
            .map(|ci| format!("\t{:?}\n", ci))
            .collect::<String>();
        write!(f, "[\n{}\n]", data_str)
    }
}

impl std::ops::Index<usize> for BestList {
    type Output = ChartInfoResponse;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

struct ChartInfo {
    idNum: String,
    diff: i32,
    tp: String,
    achievement: f32,
    ra: i32,
    comboId: i32,
    scoreId: i32,
    title: String,
    ds: f32,
    lv: String,
}

impl ChartInfo {
    fn new(
        idNum: String,
        diff: i32,
        tp: String,
        achievement: f32,
        ds: f32,
        comboId: i32,
        scoreId: i32,
        title: String,
        lv: String,
    ) -> ChartInfo {
        let ra = compute_ra(ds, achievement); // assume compute_ra is implemented elsewhere
        ChartInfo {
            idNum,
            diff,
            tp,
            achievement,
            ra,
            comboId,
            scoreId,
            title,
            ds,
            lv,
        }
    }

    fn from_json(data: ChartInfoResponse) -> ChartInfo {
        let rate = vec![
            "d", "c", "b", "bb", "bbb", "a", "aa", "aaa", "s", "sp", "ss", "ssp", "sss", "sssp",
        ];
        let ri = rate
            .iter()
            .position(|&r| r == data["rate"].as_str().unwrap())
            .unwrap();
        let fc = vec!["", "fc", "fcp", "ap", "app"];
        let fi = fc
            .iter()
            .position(|&f| f == data["fc"].as_str().unwrap())
            .unwrap();
        ChartInfo::new(
            "".to_string(),
            0,
            "".to_string(),
            0.0,
            0.0,
            0,
            0,
            "".to_string(),
            "".to_string(),
        )
    }
}

impl Display for ChartInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diffs = ["Beginner", "Easy", "Medium", "Hard", "Challenge"];
        write!(
            fmt,
            "{:<50}{}\t{}\t{}",
            format!("{} [{}]", self.title, self.tp),
            self.ds,
            diffs[self.diff as usize],
            self.ra
        )
    }
}

impl PartialEq for ChartInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ra == other.ra
    }
}

impl PartialOrd for ChartInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ra.cmp(&other.ra))
    }
}

fn compute_ra(ds: f32, achievement: f32) -> i32 {
    let mut base_ra = 22.4;
    if achievement < 50.0 {
        base_ra = 7.0;
    } else if achievement < 60.0 {
        base_ra = 8.0;
    } else if achievement < 70.0 {
        base_ra = 9.6;
    } else if achievement < 75.0 {
        base_ra = 11.2;
    } else if achievement < 80.0 {
        base_ra = 12.0;
    } else if achievement < 90.0 {
        base_ra = 13.6;
    } else if achievement < 94.0 {
        base_ra = 15.2;
    } else if achievement < 97.0 {
        base_ra = 16.8;
    } else if achievement < 98.0 {
        base_ra = 20.0;
    } else if achievement < 99.0 {
        base_ra = 20.3;
    } else if achievement < 99.5 {
        base_ra = 20.8;
    } else if achievement < 100.0 {
        base_ra = 21.1;
    } else if achievement < 100.5 {
        base_ra = 21.6;
    }

    let min_achievement = cmp::min(achievement, 100.5);
    return (ds * (min_achievement / 100.0) * base_ra).floor() as i32;
}
