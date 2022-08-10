use optimize_recursion::optimize_recursion;

#[optimize_recursion]
pub fn all_evil(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        2 => 1,
        3 => 3,
        // 4 is missing so tuple can not be filled directly
        5 => 1,
        6 => 1,
        7 => 1,
        // 8 can be calculated
        10 => 2,
        12 => 0,
        51 => 0, // this ones needs a special case in the while loop
        _ => all_evil(n - 2) + all_evil(n - 4) + all_evil(n - 8)
    }
}

pub fn all_evil_result(n: u32) -> u64
{
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    if n == 2 { return 1; }
    if n == 3 { return 3; }
    if n == 5 { return 1; }
    if n == 6 { return 1; }
    if n == 7 { return 1; }
    if n == 10 { return 2; }
    if n == 12 { return 0; }
    if n == 51 { return 0; }
    if 7 <= n && (n - 7) % 2 == 0
    {
        let mut tuple = [0; 4usize];
        tuple[0usize] = 1;
        tuple[1usize] = 3
        ;
        tuple[2usize] = 1;
        tuple[3usize] = 1;
        let mut i: usize = 3usize
            ;
        while 7 + ((i - 3usize) as u32) * 2 != n
        {
            i += 1;
            tuple[i % 4usize] = match 7 + ((i - 3usize) as u32) * 2
            {
                51 => 0,
                _ => tuple[(i - 1) % 4usize] + tuple
                    [(i - 2) % 4usize] + tuple[(i - 4) % 4usize]
            };
        }
        return tuple[i % 4usize];
    }
    panic!("result for argument not defined");
}

#[test]
pub fn test_shit() {}