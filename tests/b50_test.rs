use maimai_search_lib::clients::user_data;
use maimai_search_lib::config::consts::PROFILE;
use maimai_search_lib::image::python_module::test_python;
use pyo3::PyResult;

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
fn test_pyo3() -> PyResult<()> {
    test_python()
}
