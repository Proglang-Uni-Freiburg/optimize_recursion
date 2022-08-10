use optimize_recursion::optimize_recursion;

#[optimize_recursion]
pub fn foo(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        51 => 0,
        _ => foo(n - 2) + foo(n - 4)
    }
}

// 4 could be computed and that would lead to a possible eureka tuple fill
pub fn more_needed(n: u64) -> u64 {
    match n {
        1 => 1,
        3 => 3,
        5 => 5,
        6 => 6,
        _ => more_needed(n - 1) + more_needed(n - 3)
    }
}

// this has a special constant which needs to be considered
#[optimize_recursion]
pub fn constant_contradicts(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        5 => 1,
        _ => constant_contradicts(n - 1) + constant_contradicts(n - 2)
    }
}


#[optimize_recursion]
pub fn fib_match(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_match(n - 1)
            + fib_match(n - 2)
    }
}

#[optimize_recursion]
pub fn evil(n: u64) -> u64 {
    match n {
        100 => 1,
        102 => 2,
        104 => 3,
        99 => 9,
        _ => evil(n + 4) + evil(n + 2) + evil(n + 6) - 1
    }
}

#[optimize_recursion]
pub fn two_starts(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 2,
        2 => 1,
        _ => two_starts(n - 2) + 1
    }
}

#[optimize_recursion]
pub fn another(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        _ => another(n - 1) + another(n - 3) + another(n - 4)
    }
}

#[optimize_recursion]
pub fn another2(n: u64) -> u64 {
    match n {
        1000 => 0,
        1001 => 1,
        1002 => 2,
        1003 => 3,
        _ => another2(n + 1) + another2(n + 3) + another2(n + 4)
    }
}

// not possible
pub fn ifelse(n: u64) -> u64 {
    match n {
        0 => 0,
        _ => if n % 3 == 0 { ifelse(n - 3) } else { ifelse(n - 1) }
    }
}

pub fn evil_result(n: u64) -> u64 {
    if n == 100 { return 1; }
    if n == 102 { return 2; }
    if n == 104 { return 3; }
    if 100 >= n && (100 - n) % 2 == 0 {
        let mut tuple = [0; 3usize];
        tuple[0usize] = 3;
        tuple[1usize] = 2;
        tuple[2usize] = 1;
        let mut i: usize = 2usize;
        while 100 - ((i - 2usize) as u64) * 2 != n {
            i += 1;
            tuple[i % 3usize] = tuple[(i - 2) % 3usize] + tuple[(i - 1) % 3usize] + tuple[(i - 3) % 3usize] - 1;
        }
        return tuple[i % 3usize];
    }
    panic!("result for argument not defined");
}


pub fn foo_result(n: u32) -> u64 {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    if n == 2 { return 2; }
    if n == 3 { return 3; }
    if n == 51 {return 0; }
    if 2 <= n && (n - 2) % 2 == 0 {
        let mut tuple = [0; 2usize];
        tuple[0usize] = 0;
        tuple[1usize] = 2;
        let mut i: usize = 1usize;
        while 2 + ((i - 1usize) as u32) * 2 != n {
            i += 1;
            tuple[i % 2usize] =
            match 2 + ((i - 1usize) as u32) * 2 {
                _ => tuple[(i - 1) % 2usize] + tuple[(i - 2) % 2usize]
            };
        }
        return tuple[i % 2usize];
    }
    if 3 <= n && (n - 3) % 2 == 0 {
        let mut tuple = [0; 2usize];
        tuple[0usize] = 1;
        tuple[1usize] = 3;
        let mut i: usize = 1usize;
        while 3 + ((i - 1usize) as u32) * 2 != n {
            i += 1;
            tuple[i % 2usize] =
                match 3 + ((i - 1usize) as u32) * 2 {
                    51 => 0,
                    _ => tuple[(i - 1) % 2usize] + tuple[(i - 2) % 2usize]
            };
        }
        return tuple[i % 2usize];
    }
    panic!("result for argument not defined");
}

pub fn fib_result(n: u64) -> u64 {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    if 1 <= n && (n - 1) % 1 == 0 {
        let mut tuple = [0; 2usize];
        tuple[0usize] = 0;
        tuple[1usize] = 1;
        let mut i: usize = 1usize;
        while 1 + ((i - 1usize) as u64) * 1 != n {
            i += 1;
            tuple[i % 2usize] = tuple[(i - 1) % 2usize] + tuple[(i - 2) % 2usize];
        }
        return tuple[i % 2usize];
    }
    panic!("result for argument not defined");
}

#[test]
pub fn test_fib_match() {
    assert_eq!(fib_match(10), 55);
    assert_eq!(fib_match(1), 1);
    assert_eq!(fib_match(2), 1);
}

#[test]
pub fn test_fib_result() {
    assert_eq!(fib_result(10), 55);
    assert_eq!(fib_result(1), 1);
    assert_eq!(fib_result(2), 1);
}

#[test]
pub fn test_evil() {
    assert_eq!(evil(98), 5);
    assert_eq!(evil_result(98), 5);
    assert_eq!(evil(90), 41);
    assert_eq!(evil_result(90), 41);
}

#[test]
pub fn test_two_starts() {
    assert_eq!(two_starts(10), 5);
    assert_eq!(two_starts(9), 6);
}

#[test]
pub fn test_another() {
    assert_eq!(another(20), 9790);
}

#[test]
pub fn test_another2() {
    assert_eq!(another2(990), 290);
    // assert_eq!(another2(1004), 70);
}

#[test]
pub fn test_foo() {
    for i in 0..20 {
        assert_eq!(foo(i), foo_result(i))
    }
}