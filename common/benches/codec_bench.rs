use std::{
    fs,
    hint::black_box,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

use criterion::{criterion_group, criterion_main, Criterion};
use rand::{
    distributions::{DistString, Distribution},
    seq::SliceRandom,
    thread_rng, Rng,
};
use zalgo_codec_common::{decode_file, encode_file, zalgo_decode, zalgo_encode};

struct PrintableAsciiAndNewline;

impl Distribution<char> for PrintableAsciiAndNewline {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        *b" !\"#$%&'()*,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVXYZ[\\]^_`abcdefghijklmnopqrstuvxyz{|}~\n".choose(rng).unwrap() as char
    }
}

impl DistString for PrintableAsciiAndNewline {
    fn append_string<R: Rng + ?Sized>(&self, rng: &mut R, string: &mut String, len: usize) {
        string.reserve(len);
        for _ in 0..len {
            string.push(self.sample(rng));
        }
    }
}

fn bench_codec(c: &mut Criterion) {
    let string = PrintableAsciiAndNewline.sample_string(&mut thread_rng(), 100_000);

    let mut group = c.benchmark_group("codec");
    group.bench_function("encode", |b| b.iter(|| zalgo_encode(&string)));
    let encoded = zalgo_encode(&string).unwrap();
    group.bench_function("decode", |b| b.iter(|| zalgo_decode(&encoded)));
}

fn bench_file_codec(c: &mut Criterion) {
    let string = PrintableAsciiAndNewline.sample_string(&mut thread_rng(), 100_000);

    let mut orig_path = PathBuf::from_str("benches").unwrap();
    let mut encoded_path = orig_path.clone();
    let mut decoded_path = orig_path.clone();

    orig_path.push("original.txt");
    encoded_path.push("encoded.txt");
    decoded_path.push("decoded.txt");

    fs::write(&orig_path, &string).unwrap();

    let mut group = c.benchmark_group("file_codec");
    group.bench_function("encode_file", |b| {
        b.iter_custom(|iters| {
            let mut elapsed = Duration::from_secs(0);
            for _ in 0..iters {
                let start = Instant::now();
                black_box(encode_file(&orig_path, &encoded_path)).unwrap();
                let duration = start.elapsed();
                fs::remove_file(&encoded_path).unwrap();
                elapsed += duration;
            }
            elapsed
        })
    });

    fs::write(&encoded_path, zalgo_encode(&string).unwrap()).unwrap();

    group.bench_function("decode_file", |b| {
        b.iter_custom(|iters| {
            let mut elapsed = Duration::from_secs(0);
            for _ in 0..iters {
                let start = Instant::now();
                black_box(decode_file(&encoded_path, &decoded_path)).unwrap();
                let duration = start.elapsed();
                fs::remove_file(&decoded_path).unwrap();
                elapsed += duration;
            }
            elapsed
        })
    });
    fs::remove_file(orig_path).unwrap();
    // Depending on the order of the benchmarks, these files could be gone
    let _ = fs::remove_file(encoded_path).unwrap();
    let _ = fs::remove_file(decoded_path).unwrap();
}

criterion_group!(benches, bench_codec, bench_file_codec);
criterion_main!(benches);
