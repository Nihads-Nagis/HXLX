# Hi

```rust
println!("Hi LUXXXXXXXXX, just testin");
```

```rust,editable
fn main() {
    let choice = 4 ; // ← Change this to 1-4
    
    let square = match choice {
        1 => '🟥',  // Red
        2 => '🟩',  // Green
        3 => '🟦',  // Blue
        4 => '🟪',  // Purple
        _ => '⬛',  // Default
    };
    
    println!("The Selection is {}", square);  // Shows just one colored block
}
```
