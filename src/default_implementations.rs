use {TemplateSupport, TemplateCache};

use mustache::Template;

use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;

impl TemplateSupport for () {
    type Cache = ();
}

impl TemplateCache for () {
    fn handle<'a, P, F, R>(&self, _: &'a Path, _: P, _: F) -> R
    where P: FnOnce(&Template) -> R,
          F: FnOnce(&'a Path) -> Template {
        unreachable!()
    }
}

impl TemplateCache for RwLock<HashMap<PathBuf, Template>> {
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(&Template) -> R,
          F: FnOnce(&'a Path) -> Template {
        // Fast path doesn't need writer lock
        if let Some(t) = self.read().unwrap().get(path) {
            return handle(t);
        }

        // We didn't find the template, get writers lock
        let mut templates = self.write().unwrap();

        // Search again incase there was a race to compile the template
        let template = match templates.entry(path.into()) {
            Vacant(entry) => {
                let template = on_miss(path);
                entry.insert(template)
            }
            Occupied(entry) => entry.into_mut(),
        };

        handle(template)
    }
}
