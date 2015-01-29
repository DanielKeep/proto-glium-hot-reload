extern crate glium;
extern crate glutin;
extern crate typemap;

use typemap::Key;

pub type Frozen = typemap::TypeMap;
pub type ModuleFactory<Trait: ?Sized + 'static> = fn(&typemap::TypeMap, Option<Frozen>) -> Box<Trait>;

pub struct DisplayKey;
impl Key for DisplayKey { type Value = glium::Display; }

pub trait Reload {
    fn freeze(self: Box<Self>) -> Frozen;
}

pub trait Renderer: Reload {
    fn render(&mut self);
}
