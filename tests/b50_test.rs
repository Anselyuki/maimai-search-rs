use maimai_search_lib::clients::user_data;
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

    let mut draw_best = DrawBest::new(dx_best_list, sd_best_list, username);
    draw_best.draw().expect("TODO: panic message");
}

#[test]
fn test_create_draw_best() {
    let dx_best_list = BestList::new(15);
    let sd_best_list = BestList::new(35);
    let mut draw_best = DrawBest::new(dx_best_list, sd_best_list, "ansel");
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
