#[macro_use] extern crate nickel;
extern crate mustache;
extern crate nickel_mustache;

use nickel_mustache::{TemplateSupport, Render};
use nickel::{Nickel, HttpRouter};

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

struct ServerData<'a> {
    base: &'a Path
}

impl<'server> TemplateSupport for ServerData<'server> {
    // This should be unnecessary when default associated types work!
    type Cache = ();

    fn adjust_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        let adjusted = self.base.join(path);
        Cow::Owned(adjusted)
    }
}

fn main() {
    let data = ServerData {
        base: Path::new("examples/assets"),
    };

    let mut server = Nickel::with_data(data);

    server.get("/*", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("name", "World");

        return Render::render(res, "my_template", &data)
    });

    server.listen("127.0.0.1:0");
}
