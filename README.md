# Rust-Curl

A simple rust binding for curl, with a higher level rust library as well.

## Usage

Compile with make, then write some rust code with it e.g.

```
extern crate curl

fn main() {
    let req = curl::Request::new();

    println!("{}", req.get("http://www.google.co.uk"));
}
```

Then compile with rustc file.rs -L lib, where file.rs is the file name 
and lib is the directory where the .rlib and .so files are.

## Docs

Use make doc, then open doc/curl/index.html in your preferred web browser.

## Examples

A small example can be found in src/examples/example.rs, compile it with
make examples
