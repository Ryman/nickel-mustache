# nickel_mustache

Flexible [Mustache](https://mustache.github.io/) support for [nickel.rs](https://github.com/nickel-org/nickel.rs) using
[rust-mustache](https://github.com/nickel-org/rust-mustache).

```rust,no_run
#[macro_use] extern crate nickel;
extern crate nickel_mustache;
extern crate rustc_serialize;

use nickel_mustache::Render;
use nickel::{Nickel, HttpRouter};

fn main() {
    let mut server = Nickel::new();

    server.get("/*", middleware! { |_req, res|
        #[derive(RustcEncodable)]
        struct ViewData<'a> {
            name: &'a str
        }

        let data = ViewData { name: "World" };

        return Render::render(res, "examples/assets/my_template", &data)
    });

    server.listen("127.0.0.1:6767");
}
```

## Core Features

* Layout support
* Centralized path adjustments
* Optional template compilation caching

## Dependencies

You'll need to create a *Cargo.toml* that looks like this;

```toml
[package]

name = "my-nickel-app"
version = "0.0.1"
authors = ["yourname"]

[dependencies]
nickel_mustache = "*"
nickel = "*"
# Some examples require the `rustc_serialize` crate, which will
# require uncommenting the lines below
# rustc-serialize = "*"
```

You can then compile this using *Cargo build* and run it using *Cargo run*. After it's running you should visit http://localhost:6767 to see your hello world! (Note: the examples run with randomized ports, so please check the console output for the active port)

## More examples

More examples can be found [in the examples directory](/examples/), please check them out and you are welcome to log an issue if you have any trouble!
