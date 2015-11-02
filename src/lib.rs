#[macro_use]
extern crate nickel;
extern crate mustache;
extern crate rustc_serialize;

use rustc_serialize::Encodable;
use mustache::{Data, Template};

use std::borrow::Cow;
use std::path::Path;

mod default_implementations;
mod response_extension;

pub trait Render {
    type Output;

    fn render<T, P>(self, path: P, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>;

    fn render_data<P>(self, path: P, data: &Data) -> Self::Output where P: AsRef<Path>;
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

pub trait TemplateCache {
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(&Template) -> R,
          F: FnOnce(&'a Path) -> Template;
}
