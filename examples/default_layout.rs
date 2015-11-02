#[macro_use] extern crate nickel;
extern crate mustache;
extern crate nickel_mustache;

use nickel_mustache::{Render, TemplateSupport};
use nickel::{Nickel, HttpRouter};
use mustache::MapBuilder;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

struct ServerData;

impl TemplateSupport for ServerData {
    // This should be unnecessary when default associated types work!
    type Cache = ();

    fn default_layout(&self) -> Option<Cow<Path>> {
        let layout = Path::new("examples/assets/default_layout");
        Some(Cow::Borrowed(layout))
    }
}
fn main() {
    let mut server = Nickel::with_data(ServerData);

    server.get("/with_helpers", middleware! { |_req, res|
        let data = MapBuilder::new()
                              .insert_str("name", "World")
                              .insert_fn("helper", |text| {
                                  format!("<b>{}</b>", text.trim())
                              }).build();

        return res.render_data("examples/assets/my_template", &data)
    });


    server.get("/no_helpers", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("name", "World");

        return Render::render(res, "examples/assets/my_template", &data)
    });

    server.listen("127.0.0.1:0");
}
