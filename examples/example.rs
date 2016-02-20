#[macro_use] extern crate nickel;
extern crate nickel_mustache;
extern crate rustc_serialize;

use nickel_mustache::Render;
use nickel::{Nickel, HttpRouter};

use std::collections::HashMap;

fn main() {
    let mut server = Nickel::new();

    server.get("/map_of_values", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("name", "World");

        return Render::render(res, "examples/assets/my_template", &data)
    });

    // You can also use any type which is `Encodable`
    server.get("/*", middleware! { |_req, res|
        #[derive(RustcEncodable)]
        struct ViewData<'a> {
            name: &'a str
        }

        let data = ViewData { name: "Typed World" };

        return Render::render(res, "examples/assets/my_template", &data)
    });

    server.listen("127.0.0.1:0");
}
