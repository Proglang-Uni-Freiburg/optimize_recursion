use std::collections::HashMap;
use std::time::Instant;

const ITERATIONS: u128 = 10_000;
const N_BASE: u64 = 15;

pub fn twopower(n: u64) -> u64 {
    match n {
        0 => 1,
        _ => twopower(n - 1) + twopower(n - 1)
    }
}

pub fn twopower_memo(n: u64) -> u64 {
    pub fn twopower(n: u64, memo: &mut HashMap<u64, u64>) -> u64 {
        if !memo.contains_key(&n) {
            let result = match n {
                0 => 1,
                _ => twopower(n-1, memo) + twopower(n-1, memo)
            };
            memo.insert(n, result);
        }
        memo[&n]
    }
    twopower(n, &mut HashMap::new())
}

pub fn twopower_iter(n: u64) -> u64 {
    let mut v = 1;
    let mut i = 0;
    while i != n {
        i += 1;
        v = v + v;
    }
    v
}

#[test]
pub fn test_twopower() {
    assert_eq!(twopower(10), 1024);
    assert_eq!(twopower_memo(10), 1024);
    assert_eq!(twopower_iter(10), 1024);
}

#[test]
pub fn test_speed() {
    let n = N_BASE;
    println!("running speedtest for twopower with ITERATIONS={}", ITERATIONS);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        twopower(n);
    }
    println!("base function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, n);
    let n = 50;
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        twopower_memo(n);
    }
    println!("memo function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, n);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        twopower_iter(n);
    }
    println!("optimized function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, n);
}