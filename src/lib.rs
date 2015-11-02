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

    fn render_with_layout<T, P, L>(self, path: P, layout: L, data: &T) -> Self::Output
    where T: Encodable,
          P: AsRef<Path>,
          L: AsRef<Path>;

    fn render_data<P>(self, path: P, data: &Data) -> Self::Output where P: AsRef<Path>;

    fn render_data_with_layout<P, L>(self, path: P, layout: L, data: &Data) -> Self::Output
    where P: AsRef<Path>,
          L: AsRef<Path>;
}

pub trait TemplateSupport {
    type Cache: TemplateCache;

    fn cache(&self) -> Option<&Self::Cache> {
        None
    }

    fn adjust_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Cow::Borrowed(path)
    }

    fn adjust_layout_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Cow::Borrowed(path)
    }

    fn default_layout(&self) -> Option<Cow<Path>> {
        None
    }
}

pub trait TemplateCache {
    fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
    where P: FnOnce(Result<&Template, CompileError>) -> R,
          F: FnOnce(&'a Path) -> Result<Template, CompileError>;
}

type CompileError = String;
