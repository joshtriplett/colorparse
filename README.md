# colorparse

`colorparse::parse` parses a color configuration string (in Git syntax)
into an `ansi_term::Style`:

# Examples

```rust
if let Ok(color) = colorparse::parse("bold red blue") {
    println!("{}", color.paint("Bold red on blue"));
}
```

```rust
let hyperlink_style = colorparse::parse("#0000ee ul").unwrap();
```
