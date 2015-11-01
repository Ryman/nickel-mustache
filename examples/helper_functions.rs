#[macro_use] extern crate nickel;
extern crate mustache;
extern crate nickel_mustache;

use nickel_mustache::Render;
use nickel::{Nickel, HttpRouter};
use mustache::MapBuilder;

fn main() {
    let mut server = Nickel::new();

    server.get("/*", middleware! { |_req, res|
        let data = MapBuilder::new()
                              .insert_str("name", "World")
                              .insert_fn("helper", |text| {
                                  format!("<b>{}</b>", text.trim())
                              }).build();

        return res.render_data("examples/assets/my_template", &data)
    });

    server.listen("127.0.0.1:0");
}
