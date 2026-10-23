//! Throughput of parallel ZIP creation over a small generated corpus.

use std::fs;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use shellzip_core::{CompressOpts, NopProgress, add};

fn bench_zip_threads(c: &mut Criterion) {
    let tmp = tempfile::tempdir().expect("temp dir");
    let src = tmp.path().join("src");
    fs::create_dir(&src).expect("src");
    for index in 0..8 {
        let byte = b'a' + u8::try_from(index).unwrap_or(b'a');
        fs::write(src.join(format!("f{index}.txt")), vec![byte; 64 * 1024]).expect("file");
    }
    let bytes = 8 * 64 * 1024;
    let mut group = c.benchmark_group("zip_add");
    group.sample_size(10);
    group.throughput(Throughput::Bytes(bytes));
    for threads in [1u16, 4] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, threads| {
                b.iter(|| {
                    let archive = tmp.path().join(format!("out-{threads}.zip"));
                    let _ = fs::remove_file(&archive);
                    let opts = CompressOpts {
                        level: 1,
                        threads: *threads,
                        ..CompressOpts::default()
                    };
                    add(
                        black_box(&archive),
                        black_box(std::slice::from_ref(&src)),
                        &opts,
                        &NopProgress,
                    )
                    .expect("add");
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_zip_threads);
criterion_main!(benches);
