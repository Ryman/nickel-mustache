#[macro_use] extern crate nickel;
extern crate nickel_mustache;

use nickel_mustache::Render;
use nickel::{Nickel, HttpRouter};

use std::collections::HashMap;

fn main() {
    let mut server = Nickel::new();

    server.get("/*", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("name", "World");

        return Render::render(res, "examples/assets/my_template", &data)
    });

    server.listen("127.0.0.1:0");
}
