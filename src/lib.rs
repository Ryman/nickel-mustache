#![allow(unused_variables)]

#[macro_use]
extern crate nickel;
extern crate mustache;
extern crate rustc_serialize;

use rustc_serialize::Encodable;
use mustache::Template;
use nickel::{Response, MiddlewareResult, Halt};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::Entry::*;

pub trait Render {
    type Output;

    fn render<T, P>(self, path: P, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>;
}

pub trait TemplateSupport {
    type Cache: TemplateCache;
    fn cache(&self) -> Option<&Self::Cache> {
        None
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
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
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
          P: AsRef<Path>
    {
        let sd = self.server_data();
        let path = path.as_ref();
        let render = |template: &Template| {
            let mut stream = try!(self.start());
            match template.render(&mut stream, data) {
                Ok(()) => Ok(Halt(stream)),
                Err(e) => stream.bail(format!("Problem rendering template: {:?}", e)),
            }
        };

        let compile = |path| {
            mustache::compile_path(path).unwrap()
            // .map_err(|e| format!("Failed to compile template '{}': {:?}",
            //             path, e))
        };

        if let Some(cache) = sd.cache() {
            return cache.handle(path, render, compile);
        }

        let template = compile(path);
        render(&template)
    }
}
