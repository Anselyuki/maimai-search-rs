use std::ops::Index;

use maimai_search_lib::clients::song_data::entity::Song;
use maimai_search_lib::clients::song_data::search_songs_by_id;
use maimai_search_lib::clients::user_data;
use maimai_search_lib::clients::user_data::entity::{ChartInfoResponse, ChartRate, LevelLabel};
use maimai_search_lib::config::consts::{CONFIG_PATH, PROFILE};
use maimai_search_lib::image::maimai_best_50::{BestList, DrawBest};
use maimai_search_lib::utils::image::{compute_ra, get_ra_pic, string_to_half_width};

#[test]
fn get_b50_data() {
    let username = &PROFILE
        .remote_api
        .maimaidxprober
        .username
        .clone()
        .unwrap_or("AnselYuki".to_string());
    let resp = user_data::get_b50_data(username.as_str()).expect("TODO: panic message");
    let rating = resp.rating;
    // 查我自己的 Rating,这总不能降回去吧.jpg
    dbg!(&rating);

    let mut charts = resp.charts.sd;
    charts.extend(resp.charts.dx);
    for chart in charts {
        println!("{}", chart.to_string());
    }

    assert!(rating > 12900)
}

#[test]
fn test_compute_ra() {
    let achievement = 100.8692f32;
    let ds = 12.8f32;
    let ra = compute_ra(ds, achievement);
    println!("{}", ra)
}

#[test]
fn test_string_to_half_width() {
    let input_string = "Ｒｕｓｔ　语言";
    let output_string = string_to_half_width(&input_string);
    println!("{}", output_string); // 输出 "Rust 语言"
    assert_eq!(output_string, "Rust 语言")
}

#[test]
fn test_draw_best() {
    let resp = user_data::get_b50_data("Ashlof").expect("TODO: panic message");
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
    draw_best.draw().expect("TODO: panic message");
}

#[test]
fn test_create_draw_best() {
    let mut dx_best_list = BestList::new(15);
    let mut sd_best_list = BestList::new(35);
    let pandora = ChartInfoResponse {
        achievements: 101.0,
        ds: 15.0,
        dx_score: 1791,
        fc: "APp".to_string(),
        fs: "FSDp".to_string(),
        level: "15".to_string(),
        level_index: 3,
        level_label: LevelLabel::ReMaster,
        ra: 288,
        rate: ChartRate::SSSP,
        song_id: 834,
        title: "PANDORA PARADOXXX".to_string(),
        song_type: "SD".to_string(),
    };
    for _ in 0..35 {
        sd_best_list.push(pandora.clone());
        dx_best_list.push(pandora.clone());
    }
    let mut draw_best = DrawBest::new(sd_best_list, dx_best_list, "无声的安木");
    draw_best.draw().expect("TODO: panic message");
}

#[test]
fn create_top_b50() {
    let mut dx_best_list = BestList::new(15);
    let mut sd_best_list = BestList::new(35);
    let sd_songs_id = [
        (834, LevelLabel::ReMaster),
        (799, LevelLabel::ReMaster),
        (803, LevelLabel::ReMaster),
        (833, LevelLabel::ReMaster),
        (834, LevelLabel::Master),
        (11311, LevelLabel::Master),
        (22, LevelLabel::ReMaster),
        (227, LevelLabel::ReMaster),
        (456, LevelLabel::Master),
        (571, LevelLabel::Master),
        (643, LevelLabel::Master),
        (746, LevelLabel::Master),
        (773, LevelLabel::Master),
        (779, LevelLabel::Master),
        (799, LevelLabel::Master),
        (812, LevelLabel::ReMaster),
        (825, LevelLabel::ReMaster),
        (833, LevelLabel::Master),
        (11102, LevelLabel::Master),
        (11223, LevelLabel::Master),
        (365, LevelLabel::ReMaster),
        (496, LevelLabel::Master),
        (11026, LevelLabel::Master),
        (11106, LevelLabel::Master),
        (11165, LevelLabel::Master),
        (11222, LevelLabel::Master),
        (11235, LevelLabel::Master),
        (11364, LevelLabel::Master),
        (11374, LevelLabel::Master),
        (365, LevelLabel::Master),
        (825, LevelLabel::Master),
        (844, LevelLabel::Master),
        (852, LevelLabel::Master),
        (11029, LevelLabel::Master),
        (11103, LevelLabel::Master),
    ];
    let sd_songs: Vec<(Song, LevelLabel)> = sd_songs_id
        .iter()
        .map(|(id, label)| (search_songs_by_id(*id).unwrap(), label.clone()))
        .collect();
    for (song, label) in sd_songs {
        let ds = match label {
            LevelLabel::Basic => *song.ds.index(0),
            LevelLabel::Advanced => *song.ds.index(1),
            LevelLabel::Expert => *song.ds.index(2),
            LevelLabel::Master => *song.ds.index(3),
            LevelLabel::ReMaster => *song.ds.index(4),
        };
        sd_best_list.push(ChartInfoResponse {
            achievements: 101.0,
            ds,
            dx_score: 0,
            fc: "APp".to_string(),
            fs: "FSDp".to_string(),
            level: (song.level.last().unwrap()).parse().unwrap(),
            level_index: 0,
            level_label: label,
            ra: compute_ra(ds, 101.0),
            rate: ChartRate::SSSP,
            song_id: song.id as i32,
            title: song.title.clone(),
            song_type: "".to_string(),
        })
    }

    let dx_songs_id = [
        (11379, LevelLabel::Master),
        (11394, LevelLabel::Master),
        (11389, LevelLabel::Master),
        (11391, LevelLabel::Master),
        (11388, LevelLabel::Master),
        (11392, LevelLabel::Master),
        (11378, LevelLabel::Master),
        (11446, LevelLabel::Master),
        (1085, LevelLabel::Master),
        (11385, LevelLabel::Master),
        (11386, LevelLabel::Master),
        (11576, LevelLabel::Master),
        (11405, LevelLabel::Master),
        (11426, LevelLabel::Master),
        (11398, LevelLabel::Master),
    ];
    let dx_songs: Vec<(Song, LevelLabel)> = dx_songs_id
        .iter()
        .map(|(id, label)| (search_songs_by_id(*id).unwrap(), label.clone()))
        .collect();
    for (song, label) in dx_songs {
        let ds = match label {
            LevelLabel::Basic => *song.ds.index(0),
            LevelLabel::Advanced => *song.ds.index(1),
            LevelLabel::Expert => *song.ds.index(2),
            LevelLabel::Master => *song.ds.index(3),
            LevelLabel::ReMaster => *song.ds.index(4),
        };
        dx_best_list.push(ChartInfoResponse {
            achievements: 101.0,
            ds,
            dx_score: 0,
            fc: "APp".to_string(),
            fs: "FSDp".to_string(),
            level: (song.level.last().unwrap()).parse().unwrap(),
            level_index: 0,
            level_label: label,
            ra: compute_ra(ds, 101.0),
            rate: ChartRate::SSSP,
            song_id: song.id as i32,
            title: song.title.clone(),
            song_type: "".to_string(),
        })
    }
    let mut draw_best = DrawBest::new(sd_best_list, dx_best_list, "无声的安木");
    draw_best.draw().expect("TODO: panic message");
}

#[test]
fn test_rating_pic_url() {
    let pic_name = get_ra_pic(14961);
    let resource = CONFIG_PATH
        .join("resource")
        .join("mai")
        .join("pic")
        .join(pic_name);
    open::that(resource.clone()).expect("TODO: panic message");
    assert!(resource.exists())
}
