use std::collections::HashMap;
use std::time::Instant;
use optimize_recursion::optimize_recursion;

const N_BASE: i64 = 30;
const N: i64 = 10_000;
const ITERATIONS: u128 = 100;

// fib functions is a bit different nice thing about it: result is always 0 1 or -1

pub fn foo_base(n: i64) -> i64 {
    match n {
        0 => 0,
        1 => 1,
        _ => foo_base(n - 1) - foo_base(n - 2)
    }
}

pub fn foo_incremental(n: i64) -> i64 {
    fn foo(n: i64) -> (i64, i64) {
        match n {
            0 => (0, 1000), // second argument is not relevant
            1 => (1, 0),
            _ => {
                let r = foo(n - 1);
                (r.0 - r.1, r.0)
            }
        }
    }
    foo(n).0
}

#[optimize_recursion]
pub fn foo_optimize(n: i64) -> i64 {
    match n {
        0 => 0,
        1 => 1,
        _ => foo_optimize(n - 1) - foo_optimize(n - 2)
    }
}

pub fn foo_optimize_result(n: i64) -> i64 {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    if 1 <= n && (n - 1) % 1 == 0 {
        let mut tuple = [0; 2usize];
        tuple[0usize] = 0;
        tuple[1usize] = 1;
        let mut i: usize = 1usize;
        while 1 + ((i - 1usize) as i64) * 1 != n {
            i += 1;
            tuple[i % 2usize] = match 1 + ((i - 1usize) as i64) * 1
            { _ => tuple[(i - 1) % 2usize] - tuple[(i - 2) % 2usize] };
        }
        return tuple[i % 2usize];
    }
    panic!("result for argument not defined");
}

pub fn foo_iter(n: i64) -> i64 {
    if n == 0 {
        return 0;
    }
    let mut tuple = [0, 1];
    let mut i: usize = 1;
    while i as i64 != n {
        i += 1;
        tuple[i % 2] = tuple[(i - 1) % 2] - tuple[(i - 2) % 2]
    }
    tuple[i % 2]
}

pub fn foo_iter_optimized(n: i64) -> i64 {
    if n == 0 {
        return 0;
    }
    let mut tuple = (0, 1);
    let mut i = 1;
    while i != n {
        i += 1;
        tuple = (tuple.1, tuple.1 - tuple.0);
    }
    tuple.1
}
pub fn foo_memo_hashmap(n: i64) -> i64 {
    pub fn foo(n: i64, memo: &mut HashMap<i64, i64>) -> i64 {
        if !memo.contains_key(&n) {
            let result = match n {
                0 => 0,
                1 => 1,
                _ => foo(n - 1, memo) - foo(n - 2, memo)
            };
            memo.insert(n, result);
        }
        memo[&n]
    }
    foo(n, &mut HashMap::new())
}

pub fn foo_vec(n: i64) -> i64 {
    pub fn foo(n: usize, memo: &mut [Option<i64>]) -> i64 {
        if let Some(result) = memo[n] {
            result
        } else {
            memo[n] = Some(match n {
                0 => 0,
                1 => 1,
                _ => foo(n - 1, memo) - foo(n - 2, memo)
            });
            memo[n].unwrap()
        }
    }
    foo(n as usize, &mut vec![None; n as usize + 1])
}

pub fn foo_memo_array(memo: &mut [i64], n: usize) -> i64 {
    if memo[n] != 1000 {
        memo[n]
    } else {
        memo[n] = match n {
            0 => 0,
            1 => 1,
            _ => foo_memo_array(memo, n - 1) - foo_memo_array(memo, n - 2)
        };
        memo[n]
    }
}

#[test]
pub fn test_equal() {
    for i in 0..10 {
        assert_eq!(foo_base(i), foo_iter_optimized(i));
    }
}

#[test]
pub fn test_speed() {
    println!("running speedtest for foo with ITERATIONS={}", ITERATIONS);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_base(N_BASE);
    }
    println!("base function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N_BASE);

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_optimize(N);
    }
    println!("optimized (macro) function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_iter_optimized(N);
    }
    println!("optimized iterative function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_memo_hashmap(N);
    }
    println!("function with HashMap memo took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_vec(N);
    }
    println!("function with Vec memo and Option type took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_incremental(N);
    }
    println!("incremental function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_memo_array(&mut [1000; (N + 1) as usize], N as usize);
    }
    println!("function with Array memo (1000 = not calculated) took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        foo_iter(N);
    }
    println!("iterative function took average: {} nano seconds for n={}", start.elapsed().as_nanos() / ITERATIONS, N);
}