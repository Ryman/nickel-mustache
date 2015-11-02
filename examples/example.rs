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

    server.listen("127.0.0.1:0");
}
