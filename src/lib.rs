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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::cell::Cell;
    use mustache::{self, Template};

    use super::*;

    struct Foo {
        use_cache: bool,
        cache: FooCacher
    }

    impl Foo {
        fn new() -> Foo {
            Foo {
                use_cache: true,
                cache: FooCacher::new()
            }
        }
    }

    struct FooCacher {
        called: Cell<usize>,
        fake_cache_hit: bool,
    }

    impl FooCacher {
        fn new() -> FooCacher {
            FooCacher {
                called: Cell::new(0),
                fake_cache_hit: false
            }
        }
    }

    impl TemplateSupport for Foo {
        type Cache = FooCacher;

        fn cache(&self) -> Option<&Self::Cache> {
            if self.use_cache {
                Some(&self.cache)
            } else {
                None
            }
        }
    }

    impl TemplateCache for FooCacher {
        fn handle<'a, P, F, R>(&self, path: &'a Path, handle: P, on_miss: F) -> R
        where P: FnOnce(&Template) -> R,
              F: FnOnce(&'a Path) -> Template {
            let val = self.called.get();
            self.called.set(val + 1);

            let template = if self.fake_cache_hit {
                mustache::compile_str("")
            } else {
                on_miss(path)
            };

            handle(&template)
        }
    }

    mod cache {
        use std::path::Path;

        use super::Foo;
        use super::super::with_template;

        #[test]
        fn called() {
            let path = Path::new("examples/assets/my_template");
            let data = Foo::new();

            with_template(&path, &data, |_| ());
            assert_eq!(data.cache.called.get(), 1);
            with_template(&path, &data, |_| ());
            assert_eq!(data.cache.called.get(), 2);
        }

        #[test]
        fn used() {
            let path = Path::new("fake_file");
            let mut data = Foo::new();

            data.cache.fake_cache_hit = true;
            // This would try to compile the fake path if the cache doesn't pretend to hit.
            with_template(&path, &data, |_| ());
        }

        #[test]
        #[should_panic(expected = "No such file or directory")]
        fn sanity() {
            let path = Path::new("fake_file");
            let mut data = Foo::new();

            data.cache.fake_cache_hit = false;
            // If this doesn't panic, then the `cache_used` test isn't actually doing a valid test.
            with_template(&path, &data, |_| ());
        }

        #[test]
        fn ignored_when_none() {
            let path = Path::new("examples/assets/my_template");
            let mut data = Foo::new();
            data.use_cache = false;

            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());
            with_template(&path, &data, |_| ());

            assert_eq!(data.cache.called.get(), 0);
        }
    }
}
