#![feature(test)]

extern crate test;
extern crate gimli_hash;

use test::Bencher;
use gimli_hash::GimliHash;


#[bench]
fn bench_gimlihash_4096(b: &mut Bencher) {
    let data = vec![254; 4096];
    b.bytes = data.len() as u64;

    b.iter(|| {
        let mut res = [0; 32];
        let mut hasher = GimliHash::default();
        hasher.update(&data);
        hasher.finalize(&mut res);
    });
}
