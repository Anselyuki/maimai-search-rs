use criterion::{black_box, criterion_group, criterion_main, Criterion};

use maimai_search_lib::service::client::DXProberClient;

#[warn(unused_parens)]
fn dx_benchmark(c: &mut Criterion) {
    c.bench_function("ID检索", |b| {
        b.iter(|| black_box(DXProberClient::search_songs_by_id(11571)))
    })
    .bench_function("Title检索", |b| {
        b.iter(|| (black_box(DXProberClient::search_songs_by_title("ヒビカセ", 5))))
    });
}

criterion_group!(benches, dx_benchmark);
criterion_main!(benches);
