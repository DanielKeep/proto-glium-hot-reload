#![allow(unstable)]
#![feature(plugin)]

#[macro_use] extern crate glium;
#[plugin] extern crate glium_macros;
extern crate glutin;
extern crate typemap;
extern crate sandbox_abi;

use std::dynamic_lib::DynamicLibrary;
use typemap::TypeMap;
use sandbox_abi::{DisplayKey, Frozen, Renderer};

fn main() {
    let mut renderer_lib = None;
    let mut renderer_com = None;
    let mut renderer_frozen = None;

    // Create display outside renderer.
    let display = {
        use glium::DisplayBuild;
        glutin::WindowBuilder::new()
            .build_glium()
            .unwrap()
    };

    println!("DisplayKey type_id: {:?}", ::std::any::TypeId::of::<DisplayKey>());

    let mut deps = TypeMap::new();
    deps.insert::<DisplayKey>(display.clone());
    let deps = deps;

    // the main loop
    // each cycle will draw once
    loop {
        let exit = reload_renderer(
            &mut renderer_lib,
            &mut renderer_com,
            &mut renderer_frozen,
            &deps,
            |: renderer| {
                use std::old_io::timer;
                use std::time::Duration;

                // drawing a frame
                renderer.render();

                // sleeping for some time in order not to use up too much CPU
                timer::sleep(Duration::milliseconds(17));

                // polling and handling the events received by the window
                let mut exit = false;
                let mut reload = ReloadAction::None;
                for event in display.poll_events() {
                    match event {
                        glutin::Event::Closed => exit = true,
                        glutin::Event::ReceivedCharacter('r') => reload = ReloadAction::Reload,
                        _ => ()
                    }
                }
                (exit, reload)
            }
        );
        if exit { break }
    }
}

fn find_latest_lib(path: &Path, name: &str, ext: Option<&str>) -> Result<Option<Path>, String> {
    use std::old_io::fs;
    use std::old_io::fs::PathExtensions;

    assert_eq!(path.is_dir(), true);
    assert!(name.len() > 0);

    let lib_prefix = format!("{}{}", ::std::os::consts::DLL_PREFIX, name);
    let lib_ext = ext.unwrap_or(::std::os::consts::DLL_EXTENSION);

    let mut newest = None;

    for child in fs::readdir(path).unwrap().into_iter().filter(|p| p.is_file()) {
        let mtime = {
            match child.filename_str() {
                Some(filename) => if !filename.starts_with(&*lib_prefix) {
                    continue
                },
                None => {
                    continue
                }
            }
            if child.extension_str() != Some(lib_ext) {
                continue
            }

            match child.stat() {
                Err(err) => return Err(format!("{}", err)),
                Ok(stat) => {
                    let mtime = stat.modified;
                    if let Some(&(ref cur_mtime, _)) = newest.as_ref() {
                        if mtime <= *cur_mtime { continue }
                    }
                    mtime
                }
            }
        };

        newest = Some((mtime, child));
    }

    Ok(newest.map(|(_,p)| p))
}

enum ReloadAction {
    None,
    Reload,
}

fn reload_renderer<R, F>(
    lib: &mut Option<DynamicLibrary>,
    com: &mut Option<Box<Renderer + 'static>>,
    frozen: &mut Option<Frozen>,
    deps: &TypeMap,
    blk: F
) -> R where F: FnOnce(&mut Renderer) -> (R, ReloadAction) {
    use sandbox_abi::ModuleFactory;

    let (r, action) = {
        let com_mut: &mut Renderer = match *com {
            Some(ref mut com) => &mut **com,
            None => {
                let lib_ref = match *lib {
                    Some(ref lib) => lib,
                    None => {
                        println!("reload_renderer: reloading dylib...");
                        let path = find_latest_lib(&Path::new("target/deps"), "sandbox_render", None).unwrap().unwrap();
                        println!("- using: {:?}", path);
                        *lib = Some(DynamicLibrary::open(Some(&path)).unwrap());
                        lib.as_ref().unwrap()
                    }
                };

                let factory: ModuleFactory<Renderer + 'static> = unsafe {
                    ::std::mem::transmute(lib_ref.symbol::<()>("module_factory").unwrap())
                };

                *com = {
                    println!("reload_renderer: recreating component...");
                    Some(factory(deps, ::std::mem::replace(&mut *frozen, None)))
                };

                &mut **com.as_mut().unwrap()
            }
        };

        blk(com_mut)
    };

    match action {
        ReloadAction::None => (),
        ReloadAction::Reload => {
            println!("reload_renderer: freezing component...");
            if let Some(com) = ::std::mem::replace(&mut *com, None) {
                *frozen = Some(com.freeze());
            }

            *lib = None;
        },
    }

    r
}
