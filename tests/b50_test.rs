use maimai_search_lib::clients::user_data;
use maimai_search_lib::clients::user_data::entity::{ChartInfoResponse, ChartRate, LevelLabel};
use maimai_search_lib::config::consts::{CONFIG_PATH, PROFILE};
use maimai_search_lib::image::maimai_best_50::{BestList, DrawBest};
use maimai_search_lib::image::utils::{compute_ra, get_ra_pic, string_to_half_width};

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
    let username = &PROFILE
        .remote_api
        .maimaidxprober
        .username
        .clone()
        .unwrap_or("AnselYuki".to_string());
    let resp = user_data::get_b50_data(username.as_str()).expect("TODO: panic message");
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
    let dx_best_list = BestList::new(15);
    let mut sd_best_list = BestList::new(35);
    sd_best_list.push(ChartInfoResponse {
        achievements: 100.8692,
        ds: 13.4,
        dx_score: 1791,
        fc: "fcp".to_string(),
        fs: "fs".to_string(),
        level: "12+".to_string(),
        level_index: 3,
        level_label: LevelLabel::Master,
        ra: 288,
        rate: ChartRate::SSSP,
        song_id: 547,
        title: "私の中の幻想的世界観及びその顕現を想起させたある現実での出来事に関する一考察"
            .to_string(),
        song_type: "SD".to_string(),
    });
    sd_best_list.push(ChartInfoResponse {
        achievements: 97.5604,
        ds: 13.2,
        dx_score: 2018,
        fc: "fcp".to_string(),
        fs: "fs".to_string(),
        level: "13".to_string(),
        level_index: 3,
        level_label: LevelLabel::Master,
        ra: 257,
        rate: ChartRate::S,
        song_id: 11266,
        title: "キラメキ居残り大戦争".to_string(),
        song_type: "DX".to_string(),
    });
    let mut draw_best = DrawBest::new(sd_best_list, dx_best_list, "ANSEL");
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
