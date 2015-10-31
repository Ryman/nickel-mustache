#[macro_use]
extern crate nickel;
extern crate mustache;
extern crate rustc_serialize;

use rustc_serialize::Encodable;
use mustache::{Data, Template};
use nickel::{Response, MiddlewareResult, Halt};

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;

pub trait Render {
    type Output;

    fn render<T, P>(self, path: P, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>;

    fn render_data<P>(self, path: P, data: &Data) -> Self::Output
    where P: AsRef<Path>;
}

pub trait TemplateSupport {
    type Cache: TemplateCache;

    fn cache(&self) -> Option<&Self::Cache> {
        None
    }

    fn adjust_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Cow::Borrowed(path)
    }
}

impl TemplateSupport for () {
    type Cache = ();
}

pub trait TemplateCache {
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(&Template) -> R,
          F: FnOnce(&'a Path) -> Template;
}

impl TemplateCache for () {
    fn handle<'a, P, F, R>(&self, _: &'a Path, _: P, _: F) -> R
    where P: FnOnce(&Template) -> R,
          F: FnOnce(&'a Path) -> Template
    {
        unreachable!()
    }
}

impl TemplateCache for RwLock<HashMap<PathBuf, Template>> {
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(&Template) -> R,
          F: FnOnce(&'a Path) -> Template
    {
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

impl<'a, 'mw, D> Render for Response<'mw, D>
where D: TemplateSupport {
    type Output = MiddlewareResult<'mw, D>;

    fn render<T, P>(self, path: P, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path> {
        with_template(path.as_ref(),
                             self.server_data(),
                             |template| {
                                 let mut stream = try!(self.start());
                                 match template.render(&mut stream, data) {
                                     Ok(()) => Ok(Halt(stream)),
                                     Err(e) => stream.bail(format!("Problem rendering template: {:?}", e)),
                                 }
                             })
    }

    fn render_data<P>(self, path: P, data: &Data) -> Self::Output
    where P: AsRef<Path> {
        with_template(path.as_ref(),
                             self.server_data(),
                             |template| {
                                 let mut stream = try!(self.start());
                                 template.render_data(&mut stream, data);
                                 Ok(Halt(stream))
                             })
    }
}

fn with_template<F, D, T>(path: &Path, data: &D, f: F) -> T
where D: TemplateSupport,
      F: FnOnce(&Template) -> T {
    let path = &*data.adjust_path(path);

    let compile = |path| {
            mustache::compile_path(path).unwrap()
            // .map_err(|e| format!("Failed to compile template '{}': {:?}",
            //             path, e))
    };

    if let Some(cache) = data.cache() {
        return cache.handle(path, f, compile);
    }

    let template = compile(path);
    f(&template)
}
