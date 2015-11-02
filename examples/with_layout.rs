#[macro_use] extern crate nickel;
extern crate mustache;
extern crate nickel_mustache;

use nickel_mustache::Render;
use nickel::{Nickel, HttpRouter};
use mustache::MapBuilder;

use std::collections::HashMap;

fn main() {
    let mut server = Nickel::new();

    server.get("/with_helpers", middleware! { |_req, res|
        let data = MapBuilder::new()
                              .insert_str("name", "World")
                              .insert_fn("helper", |text| {
                                  format!("<b>{}</b>", text.trim())
                              }).build();

        return res.render_data_with_layout("examples/assets/my_template",
                                           "examples/assets/layout",
                                           &data)
    });


    server.get("/no_helpers", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("name", "World");

        return res.render_with_layout("examples/assets/my_template",
                                      "examples/assets/layout",
                                      &data)
    });

    server.listen("127.0.0.1:0");
}
