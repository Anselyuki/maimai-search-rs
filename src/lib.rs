pub mod clients;
pub mod config;
mod db;
pub mod service;
mod utils;

#[cfg(test)]
mod tests {
    use std::ops::Index;

    use crate::clients::song_data::search_songs_by_id;
    use crate::clients::user_data;
    use crate::clients::user_data::entity::{ChartInfoResponse, ChartRate, LevelLabel};
    use crate::config::consts::PROFILE;
    use crate::service::maimai_best_50::{BestList, DrawBest};
    use crate::utils::image::compute_ra;

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
        assert!(rating > 12900)
    }

    #[test]
    fn test_draw_best() {
        let resp = user_data::get_b50_data("AnselYuki").expect("TODO: panic message");
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
    fn create_top_b50() {
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
        let sd_best_list = create_chart_info_responses(&sd_songs_id, 35);
        let dx_best_list = create_chart_info_responses(&dx_songs_id, 15);
        let mut draw_best = DrawBest::new(sd_best_list, dx_best_list, "无声的安木");
        draw_best.draw().expect("TODO: panic message");
    }

    fn create_chart_info_responses(song_ids: &[(i32, LevelLabel)], size: usize) -> BestList {
        let mut chart_info_responses = BestList::new(size);
        for (song_id, level_label) in song_ids {
            if let Some(song) = search_songs_by_id(*song_id as usize) {
                let ds = match level_label {
                    LevelLabel::Basic => *song.ds.index(0),
                    LevelLabel::Advanced => *song.ds.index(1),
                    LevelLabel::Expert => *song.ds.index(2),
                    LevelLabel::Master => *song.ds.index(3),
                    LevelLabel::ReMaster => *song.ds.index(4),
                };
                chart_info_responses.push(ChartInfoResponse {
                    achievements: 101.0,
                    ds,
                    dx_score: 0,
                    fc: "APp".to_string(),
                    fs: "FSDp".to_string(),
                    level: (song.level.last().unwrap()).parse().unwrap(),
                    level_label: level_label.clone(),
                    ra: compute_ra(ds, 101.0),
                    rate: ChartRate::SSSP,
                    song_id: song.id as i32,
                    title: song.title.clone(),
                    song_type: song.song_type,
                });
            }
        }
        chart_info_responses
    }
}
