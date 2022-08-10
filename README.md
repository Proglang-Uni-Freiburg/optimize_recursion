# Rust macro to optimize recursions

## Usage
Recursive functions can be annotated an example:
```rust
#[optimize_recursion]
pub fn fib(a: u32) -> u64 {
  match a {
    0 => 0,
    1 => 1,
    _ => fib(a-1) + f(a-2)
  }
}
```
