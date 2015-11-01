#[macro_use] extern crate nickel;
extern crate mustache;
extern crate nickel_mustache;

use nickel_mustache::{TemplateSupport, Render};
use nickel::{Nickel, HttpRouter};
use mustache::Template;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use std::env;

type Cache = HashMap<PathBuf, Template>;

struct ServerData {
    use_cache: bool,
    template_cache: RwLock<Cache>
}

impl TemplateSupport for ServerData {
    type Cache = RwLock<Cache>;

    fn cache(&self) -> Option<&Self::Cache> {
        if self.use_cache {
            Some(&self.template_cache)
        } else {
            None
        }
    }
}

fn main() {
    let data = ServerData {
        use_cache: env::args().all(|arg| arg != "nocache"),
        template_cache: RwLock::new(HashMap::new())
    };

    let mut server = Nickel::with_data(data);

    server.get("/*", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("name", "World");

        return Render::render(res, "examples/assets/my_template", &data)
    });

    server.listen("127.0.0.1:0");
}
