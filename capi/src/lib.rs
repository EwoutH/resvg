// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(non_camel_case_types)]

extern crate resvg;
extern crate fern;
#[macro_use] extern crate log;

#[cfg(feature = "cairo-backend")] extern crate cairo_sys;

use std::fmt;
use std::path;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::ptr;

#[cfg(feature = "qt-backend")] use resvg::qt;
#[cfg(feature = "cairo-backend")] use resvg::cairo;
use resvg::prelude::*;

const DEFAULT_FONT_FAMILY: &str = "Times New Roman";

#[repr(C)]
pub struct resvg_options {
    pub path: *const c_char,
    pub dpi: f64,
    pub font_family: *const c_char,
    pub font_size: f64,
    pub languages: *const c_char,
    pub fit_to: resvg_fit_to,
    pub draw_background: bool,
    pub background: resvg_color,
    pub keep_named_groups: bool,
}

enum ErrorId {
    Ok = 0,
    NotAnUtf8Str,
    FileOpenFailed,
    FileWriteFailed,
    InvalidFileSuffix,
    MalformedGZip,
    InvalidSize,
    ParsingFailed,
    NoCanvas,
}

#[repr(C)]
pub struct resvg_color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[repr(C)]
pub enum resvg_fit_to_type {
    RESVG_FIT_TO_ORIGINAL,
    RESVG_FIT_TO_WIDTH,
    RESVG_FIT_TO_HEIGHT,
    RESVG_FIT_TO_ZOOM,
}

#[repr(C)]
pub struct resvg_fit_to {
    kind: resvg_fit_to_type,
    value: f32,
}

#[repr(C)]
pub struct resvg_rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
pub struct resvg_size {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct resvg_transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

#[repr(C)]
pub struct resvg_render_tree(resvg::usvg::Tree);

#[repr(C)]
pub struct resvg_handle(resvg::InitObject);


#[no_mangle]
pub extern fn resvg_init() -> *mut resvg_handle {
    let handle = Box::new(resvg_handle(resvg::init()));
    Box::into_raw(handle)
}

#[no_mangle]
pub extern fn resvg_destroy(handle: *mut resvg_handle) {
    unsafe {
        assert!(!handle.is_null());
        Box::from_raw(handle)
    };
}

#[no_mangle]
pub extern fn resvg_init_log() {
    fern::Dispatch::new()
        .format(log_format)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stderr())
        .apply().unwrap();
}

fn log_format(out: fern::FormatCallback, message: &fmt::Arguments, record: &log::Record) {
    let lvl = match record.level() {
        log::Level::Error => "Error",
        log::Level::Warn => "Warning",
        log::Level::Info => "Info",
        log::Level::Debug => "Debug",
        log::Level::Trace => "Trace",
    };

    out.finish(format_args!(
        "{} (in {}:{}): {}",
        lvl,
        record.target(),
        record.line().unwrap_or(0),
        message
    ))
}

#[no_mangle]
pub extern fn resvg_init_options(opt: *mut resvg_options) {
    unsafe {
        (*opt).path = ptr::null();
        (*opt).dpi = 96.0;
        (*opt).font_family = ptr::null();
        (*opt).font_size = 12.0;
        (*opt).languages = ptr::null();
        (*opt).fit_to = resvg_fit_to {
            kind: resvg_fit_to_type::RESVG_FIT_TO_ORIGINAL,
            value: 0.0,
        };
        (*opt).draw_background = false;
        (*opt).background.r = 0;
        (*opt).background.g = 0;
        (*opt).background.b = 0;
        (*opt).keep_named_groups = false;
    }
}

#[no_mangle]
pub extern fn resvg_parse_tree_from_file(
    file_path: *const c_char,
    opt: *const resvg_options,
    raw_tree: *mut *mut resvg_render_tree,
) -> i32 {
    let file_path = match cstr_to_str(file_path) {
        Some(v) => v,
        None => return ErrorId::NotAnUtf8Str as i32,
    };

    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    let tree = match usvg::Tree::from_file(file_path, &opt.usvg) {
        Ok(tree) => tree,
        Err(e) => return convert_error(e) as i32,
    };

    let tree_box = Box::new(resvg_render_tree(tree));
    unsafe { *raw_tree = Box::into_raw(tree_box); }

    ErrorId::Ok as i32
}

#[no_mangle]
pub extern fn resvg_parse_tree_from_data(
    data: *const c_char,
    len: usize,
    opt: *const resvg_options,
    raw_tree: *mut *mut resvg_render_tree,
) -> i32 {
    let data = unsafe { slice::from_raw_parts(data as *const u8, len) };

    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    let tree = match usvg::Tree::from_data(data, &opt.usvg) {
        Ok(tree) => tree,
        Err(e) => return convert_error(e) as i32,
    };

    let tree_box = Box::new(resvg_render_tree(tree));
    unsafe { *raw_tree = Box::into_raw(tree_box); }

    ErrorId::Ok as i32
}

#[no_mangle]
pub extern fn resvg_tree_destroy(tree: *mut resvg_render_tree) {
    unsafe {
        assert!(!tree.is_null());
        Box::from_raw(tree)
    };
}

#[cfg(feature = "qt-backend")]
#[no_mangle]
pub extern fn resvg_qt_render_to_image(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    file_path: *const c_char,
) -> i32 {
    let backend = Box::new(resvg::backend_qt::Backend);
    render_to_image(tree, opt, file_path, backend)
}

#[cfg(feature = "cairo-backend")]
#[no_mangle]
pub extern fn resvg_cairo_render_to_image(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    file_path: *const c_char,
) -> i32 {
    let backend = Box::new(resvg::backend_cairo::Backend);
    render_to_image(tree, opt, file_path, backend)
}

fn render_to_image(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    file_path: *const c_char,
    backend: Box<resvg::Render>,
) -> i32 {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let file_path = match cstr_to_str(file_path) {
        Some(v) => v,
        None => return ErrorId::NotAnUtf8Str as i32,
    };

    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    let img = backend.render_to_image(&tree.0, &opt);
    let img = match img {
        Some(img) => img,
        None => {
            return ErrorId::NoCanvas as i32;
        }
    };

    match img.save(path::Path::new(file_path)) {
        true => ErrorId::Ok as i32,
        false => ErrorId::FileWriteFailed as i32,
    }
}

#[cfg(feature = "qt-backend")]
#[no_mangle]
pub extern fn resvg_qt_render_to_canvas(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    size: resvg_size,
    painter: *mut qt::qtc_qpainter,
) {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let mut painter = unsafe { qt::Painter::from_raw(painter) };
    let size = resvg::ScreenSize::new(size.width, size.height);
    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    resvg::backend_qt::render_to_canvas(&tree.0, &opt, size, &mut painter);
}

#[cfg(feature = "cairo-backend")]
#[no_mangle]
pub extern fn resvg_cairo_render_to_canvas(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    size: resvg_size,
    cr: *mut cairo_sys::cairo_t,
) {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let cr = unsafe { cairo::Context::from_raw_none(cr) };
    let size = resvg::ScreenSize::new(size.width, size.height);

    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    resvg::backend_cairo::render_to_canvas(&tree.0, &opt, size, &cr);
}

#[cfg(feature = "qt-backend")]
#[no_mangle]
pub extern fn resvg_qt_render_to_canvas_by_id(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    size: resvg_size,
    id: *const c_char,
    painter: *mut qt::qtc_qpainter,
) {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let mut painter = unsafe { qt::Painter::from_raw(painter) };
    let size = resvg::ScreenSize::new(size.width, size.height);
    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    let id = match cstr_to_str(id) {
        Some(v) => v,
        None => return,
    };

    if id.is_empty() {
        warn!("Node with an empty ID cannot be painted.");
        return;
    }

    if let Some(node) = tree.0.node_by_id(id) {
        if let Some(bbox) = resvg::backend_qt::calc_node_bbox(&node, &opt) {
            let vbox = usvg::ViewBox {
                rect: bbox,
                aspect: usvg::AspectRatio::default(),
            };

            resvg::backend_qt::render_node_to_canvas(&node, &opt, vbox, size, &mut painter);
        } else {
            warn!("A node with '{}' ID doesn't have a valid bounding box.", id);
        }
    } else {
        warn!("A node with '{}' ID wasn't found.", id);
    }
}

#[cfg(feature = "cairo-backend")]
#[no_mangle]
pub extern fn resvg_cairo_render_to_canvas_by_id(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    size: resvg_size,
    id: *const c_char,
    cr: *mut cairo_sys::cairo_t,
) {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let id = match cstr_to_str(id) {
        Some(v) => v,
        None => return,
    };

    if id.is_empty() {
        warn!("Node with an empty ID cannot be painted.");
        return;
    }

    let cr = unsafe { cairo::Context::from_raw_none(cr) };
    let size = resvg::ScreenSize::new(size.width, size.height);

    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    if let Some(node) = tree.0.node_by_id(id) {
        if let Some(bbox) = resvg::backend_cairo::calc_node_bbox(&node, &opt) {
            let vbox = usvg::ViewBox {
                rect: bbox,
                aspect: usvg::AspectRatio::default(),
            };

            resvg::backend_cairo::render_node_to_canvas(&node, &opt, vbox, size, &cr);
        } else {
            warn!("A node with '{}' ID doesn't have a valid bounding box.", id);
        }
    } else {
        warn!("A node with '{}' ID wasn't found.", id);
    }
}

#[no_mangle]
pub extern fn resvg_get_image_size(
    tree: *const resvg_render_tree,
) -> resvg_size {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let size = tree.0.svg_node().size;

    resvg_size {
        width: size.width as u32,
        height: size.height as u32,
    }
}

#[no_mangle]
pub extern fn resvg_get_image_viewbox(
    tree: *const resvg_render_tree,
) -> resvg_rect {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    let r = tree.0.svg_node().view_box.rect;

    resvg_rect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    }
}

#[no_mangle]
pub extern fn resvg_is_image_empty(
    tree: *const resvg_render_tree,
) -> bool {
    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    tree.0.root().has_children()
}

#[cfg(feature = "qt-backend")]
#[no_mangle]
pub extern fn resvg_qt_get_node_bbox(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    id: *const c_char,
    bbox: *mut resvg_rect,
) -> bool {
    let backend = Box::new(resvg::backend_qt::Backend);
    get_node_bbox(tree, opt, id, bbox, backend)
}

#[cfg(feature = "cairo-backend")]
#[no_mangle]
pub extern fn resvg_cairo_get_node_bbox(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    id: *const c_char,
    bbox: *mut resvg_rect,
) -> bool {
    let backend = Box::new(resvg::backend_cairo::Backend);
    get_node_bbox(tree, opt, id, bbox, backend)
}

fn get_node_bbox(
    tree: *const resvg_render_tree,
    opt: *const resvg_options,
    id: *const c_char,
    bbox: *mut resvg_rect,
    backend: Box<resvg::Render>,
) -> bool {
    let id = match cstr_to_str(id) {
        Some(v) => v,
        None => {
            warn!("Provided ID is no an UTF-8 string.");
            return false;
        }
    };

    if id.is_empty() {
        warn!("Node ID must not be empty.");
        return false;
    }

    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };


    let opt = to_native_opt(unsafe {
        assert!(!opt.is_null());
        &*opt
    });

    match tree.0.node_by_id(id) {
        Some(node) => {
            if let Some(r) = backend.calc_node_bbox(&node, &opt) {
                unsafe {
                    (*bbox).x = r.x;
                    (*bbox).y = r.y;
                    (*bbox).width = r.width;
                    (*bbox).height = r.height;
                }

                true
            } else {
                false
            }
        }
        None => {
            warn!("No node with '{}' ID is in the tree.", id);
            false
        }
    }
}

#[no_mangle]
pub extern fn resvg_node_exists(
    tree: *const resvg_render_tree,
    id: *const c_char,
) -> bool {
    let id = match cstr_to_str(id) {
        Some(v) => v,
        None => {
            warn!("Provided ID is no an UTF-8 string.");
            return false;
        }
    };

    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    tree.0.node_by_id(id).is_some()
}

#[no_mangle]
pub extern fn resvg_get_node_transform(
    tree: *const resvg_render_tree,
    id: *const c_char,
    ts: *mut resvg_transform,
) -> bool {
    let id = match cstr_to_str(id) {
        Some(v) => v,
        None => {
            warn!("Provided ID is no an UTF-8 string.");
            return false;
        }
    };

    let tree = unsafe {
        assert!(!tree.is_null());
        &*tree
    };

    if let Some(node) = tree.0.node_by_id(id) {
        let mut abs_ts = resvg::utils::abs_transform(&node);
        abs_ts.append(&node.transform());

        unsafe {
            (*ts).a = abs_ts.a;
            (*ts).b = abs_ts.b;
            (*ts).c = abs_ts.c;
            (*ts).d = abs_ts.d;
            (*ts).e = abs_ts.e;
            (*ts).f = abs_ts.f;
        }

        return true;
    }

    false
}

fn cstr_to_str(text: *const c_char) -> Option<&'static str> {
    let text = unsafe {
        assert!(!text.is_null());
        CStr::from_ptr(text)
    };

    text.to_str().ok()
}

fn to_native_opt(opt: &resvg_options) -> resvg::Options {
    let mut path: Option<path::PathBuf> = None;

    if !opt.path.is_null() {
        if let Some(p) = cstr_to_str(opt.path) {
            if !p.is_empty() {
                path = Some(p.into());
            }
        }
    };

    let fit_to = match opt.fit_to.kind {
        resvg_fit_to_type::RESVG_FIT_TO_ORIGINAL => {
            resvg::FitTo::Original
        }
        resvg_fit_to_type::RESVG_FIT_TO_WIDTH => {
            assert!(opt.fit_to.value > 0.0);
            resvg::FitTo::Width(opt.fit_to.value as u32)
        }
        resvg_fit_to_type::RESVG_FIT_TO_HEIGHT => {
            assert!(opt.fit_to.value > 0.0);
            resvg::FitTo::Height(opt.fit_to.value as u32)
        }
        resvg_fit_to_type::RESVG_FIT_TO_ZOOM => {
            assert!(opt.fit_to.value > 0.0);
            resvg::FitTo::Zoom(opt.fit_to.value)
        }
    };

    let background = if opt.draw_background {
        Some(resvg::usvg::Color::new(
            opt.background.r,
            opt.background.g,
            opt.background.b,
        ))
    } else {
        None
    };

    let ff = DEFAULT_FONT_FAMILY;
    let font_family = match cstr_to_str(opt.font_family) {
        Some(v) => {
            if v.is_empty() {
                warn!("Provided 'font_family' option is empty. Fallback to '{}'.", ff);
                ff
            } else {
                v
            }
        }
        None => {
            warn!("Provided 'font_family' option is no an UTF-8 string. Fallback to '{}'.", ff);
            ff
        }
    };


    let languages_str = match cstr_to_str(opt.languages) {
        Some(v) => v,
        None => {
            warn!("Provided 'languages' option is no an UTF-8 string. Fallback to 'en'.");
            "en"
        }
    };

    let mut languages = Vec::new();
    for lang in languages_str.split(',') {
        languages.push(lang.trim().to_string());
    }

    if languages.is_empty() {
        warn!("Provided 'languages' option is empty. Fallback to 'en'.");
        languages = vec!["en".to_string()]
    }


    resvg::Options {
        usvg: usvg::Options {
            path,
            dpi: opt.dpi,
            font_family: font_family.to_string(),
            font_size: opt.font_size,
            languages,
            keep_named_groups: opt.keep_named_groups,
        },
        fit_to,
        background,
    }
}

fn convert_error(e: usvg::Error) -> ErrorId {
    match e {
        usvg::Error::InvalidFileSuffix => ErrorId::InvalidFileSuffix,
        usvg::Error::FileOpenFailed => ErrorId::FileOpenFailed,
        usvg::Error::NotAnUtf8Str => ErrorId::NotAnUtf8Str,
        usvg::Error::MalformedGZip => ErrorId::MalformedGZip,
        usvg::Error::InvalidSize => ErrorId::InvalidSize,
        usvg::Error::ParsingFailed(_) => ErrorId::ParsingFailed,
    }
}
