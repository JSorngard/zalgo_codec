use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::{
    distributions::{DistString, Distribution},
    seq::SliceRandom,
    thread_rng, Rng,
};
use zalgo_codec_common::{zalgo_decode, zalgo_encode};

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
    group.bench_function("encode", |b| {
        b.iter(|| black_box(zalgo_encode(&string)).unwrap())
    });
    let encoded = zalgo_encode(&string).unwrap();
    group.bench_function("decode", |b| {
        b.iter(|| black_box(zalgo_decode(&encoded)).unwrap())
    });
}

criterion_group!(benches, bench_codec);
criterion_main!(benches);
