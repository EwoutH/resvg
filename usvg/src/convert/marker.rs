// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::f64;

// self
use utils;
use tree;
use tree::prelude::*;
use tree::PathSegment as Segment;
use super::prelude::*;
use super::use_node;


pub fn convert(
    node: &svgdom::Node,
    segments: &[tree::PathSegment],
    state: &State,
    parent: &mut tree::Node,
    tree: &mut tree::Tree,
) {
    // `marker-*` attributes can only be set on `path`, `line`, `polyline` and `polygon`.
    match node.tag_id() {
          Some(EId::Path)
        | Some(EId::Line)
        | Some(EId::Polyline)
        | Some(EId::Polygon) => {}
        _ => return,
    }

    // `marker-*` attributes cannot be set on shapes inside the `clipPath`.
    if node.ancestors().any(|n| n.is_tag_name(EId::ClipPath)) {
        return;
    }

    let list = [
        (AId::MarkerStart, MarkerKind::Start),
        (AId::MarkerMid, MarkerKind::Middle),
        (AId::MarkerEnd, MarkerKind::End),
    ];

    for (aid, kind) in &list {
        let mut marker = None;
        for n in node.ancestors() {
            let attrs = n.attributes();
            if let Some(&AValue::FuncLink(ref link)) = attrs.get_value(*aid) {
                if link.is_tag_name(EId::Marker) {
                    marker = Some(link.clone());
                }
            }
        }

        if let Some(marker) = marker {
            // Check for recursive marker.
            if state.current_root == marker {
                continue;
            }

            resolve(node, segments, &marker, *kind, state, parent, tree);
        }
    }
}

#[derive(Clone, Copy)]
enum MarkerKind {
    Start,
    Middle,
    End,
}

enum MarkerOrientation {
    Auto,
    Angle(f64),
}

fn resolve(
    shape_node: &svgdom::Node,
    segments: &[tree::PathSegment],
    marker_node: &svgdom::Node,
    marker_kind: MarkerKind,
    state: &State,
    parent: &mut tree::Node,
    tree: &mut tree::Tree,
) {
    let stroke_scale = try_opt!(stroke_scale(shape_node, marker_node, state), ());

    let r = convert_rect(marker_node, state);
    if !r.is_valid() {
        return;
    }

    let view_box = marker_node.get_viewbox().map(|vb|
        tree::ViewBox {
            rect: vb,
            aspect: super::convert_aspect(&*marker_node.attributes()),
        }
    );

    let has_overflow = {
        let attrs = marker_node.attributes();
        let overflow = attrs.get_str_or(AId::Overflow, "hidden");
        overflow == "hidden" || overflow == "scroll"
    };

    let clip_path = if has_overflow {
        let clip_rect = if let Some(vbox) = view_box {
            Rect::new(vbox.rect.x, vbox.rect.y, vbox.rect.width, vbox.rect.height)
        } else {
            Rect::new(0.0, 0.0, r.width, r.height)
        };

        let id = use_node::gen_clip_path_id(shape_node, tree);

        let mut clip_path = tree.append_to_defs(
            tree::NodeKind::ClipPath(tree::ClipPath {
                id: id.clone(),
                units: tree::Units::UserSpaceOnUse,
                transform: tree::Transform::default(),
                clip_path: None,
            })
        );

        clip_path.append_kind(tree::NodeKind::Path(tree::Path {
            id: String::new(),
            transform: tree::Transform::default(),
            visibility: tree::Visibility::Visible,
            fill: Some(tree::Fill::default()),
            stroke: None,
            segments: utils::rect_to_path(clip_rect),
        }));

        Some(id)
    } else {
        None
    };

    let draw_marker = |x: f64, y: f64, idx: usize| {
        let mut ts = tree::Transform::new_translate(x, y);

        let angle = match convert_orientation(&*marker_node.attributes()) {
            MarkerOrientation::Auto => calc_vertex_angle(&segments, idx),
            MarkerOrientation::Angle(angle) => angle,
        };

        if !angle.is_fuzzy_zero() {
            ts.rotate(angle);
        }

        if let Some(vbox) = view_box {
            let size = Size::new(r.width * stroke_scale, r.height * stroke_scale);
            let vbox_ts = utils::view_box_to_transform(vbox.rect, vbox.aspect, size);
            let (sx, sy) = vbox_ts.get_scale();
            ts.scale(sx, sy);
        } else {
            ts.scale(stroke_scale, stroke_scale);
        }

        ts.translate(-r.x, -r.y);


        // TODO: do not create a group when no clipPath
        let mut g_node = parent.append_kind(tree::NodeKind::Group(tree::Group {
            id: String::new(),
            transform: ts,
            opacity: tree::Opacity::default(),
            clip_path: clip_path.clone(),
            mask: None,
            filter: None,
        }));

        let mut marker_state = state.clone();
        marker_state.current_root = marker_node.clone();
        super::convert_children(marker_node, &marker_state, &mut g_node, tree);

        if !g_node.has_children() {
            g_node.detach();
        }
    };

    draw_markers(&segments, marker_kind, draw_marker);
}

fn stroke_scale(
    path_node: &svgdom::Node,
    marker_node: &svgdom::Node,
    state: &State,
) -> Option<f64> {
    match marker_node.attributes().get_str_or(AId::MarkerUnits, "strokeWidth") {
        "userSpaceOnUse" => Some(1.0),
        _ => {
            let sw = path_node.resolve_length(AId::StrokeWidth, state, 1.0);
            if !(sw > 0.0) {
                None
            } else {
                Some(sw)
            }
        }
    }
}

fn draw_markers<P>(segments: &[tree::PathSegment], kind: MarkerKind, mut draw_marker: P)
    where P: FnMut(f64, f64, usize)
{
    match kind {
        MarkerKind::Start => {
            if let Some(tree::PathSegment::MoveTo { x, y }) = segments.first() {
                draw_marker(*x, *y, 0);
            }
        }
        MarkerKind::Middle => {
            let total = segments.len() - 1;
            let mut i = 1;
            while i < total {
                let (x, y) = match segments[i] {
                    tree::PathSegment::MoveTo { x, y } => (x, y),
                    tree::PathSegment::LineTo { x, y } => (x, y),
                    tree::PathSegment::CurveTo { x, y, .. } => (x, y),
                    _ => {
                        i += 1;
                        continue
                    }
                };

                draw_marker(x, y, i);

                i += 1;
            }
        }
        MarkerKind::End => {
            let idx = segments.len() - 1;
            match segments.last() {
                Some(Segment::LineTo { x, y }) => {
                    draw_marker(*x, *y, idx);
                }
                Some(Segment::CurveTo { x, y, .. }) => {
                    draw_marker(*x, *y, idx);
                }
                Some(Segment::ClosePath) => {
                    let (x, y) = get_subpath_start(segments, idx);
                    draw_marker(x, y, idx);
                }
                _ => {}
            }
        }
    }
}

fn calc_vertex_angle(segments: &[Segment], idx: usize) -> f64 {
    if idx == 0 {
        // First segment.

        debug_assert!(segments.len() > 1);

        let seg1 = segments[0];
        let seg2 = segments[1];

        match (seg1, seg2) {
            (Segment::MoveTo { x: mx, y: my }, Segment::LineTo { x, y }) => {
                calc_line_angle(mx, my, x, y)
            }
            (Segment::MoveTo { x: mx, y: my }, Segment::CurveTo { x1, y1, x, y, .. }) => {
                if mx.fuzzy_eq(&x1) && my.fuzzy_eq(&y1) {
                    calc_line_angle(mx, my, x, y)
                } else {
                    calc_line_angle(mx, my, x1, y1)
                }
            }
            _ => 0.0,
        }
    } else if idx == segments.len() - 1 {
        // Last segment.

        let seg1 = segments[idx - 1];
        let seg2 = segments[idx];

        match (seg1, seg2) {
            (_, Segment::MoveTo { .. }) => 0.0, // unreachable
            (_, Segment::LineTo { x, y }) => {
                let (px, py) = get_prev_vertex(segments, idx);
                calc_line_angle(px, py, x, y)
            }
            (_, Segment::CurveTo { x2, y2, x, y, .. }) => {
                if x2.fuzzy_eq(&x) && y2.fuzzy_eq(&y) {
                    let (px, py) = get_prev_vertex(segments, idx);
                    calc_line_angle(px, py, x, y)
                } else {
                    calc_line_angle(x2, y2, x, y)
                }
            }
            (Segment::LineTo { x, y }, Segment::ClosePath) => {
                let (nx, ny) = get_subpath_start(segments, idx);
                calc_line_angle(x, y, nx, ny)
            }
            (Segment::CurveTo { x2, y2, x, y, .. }, Segment::ClosePath) => {
                let (px, py) = get_prev_vertex(segments, idx);
                let (nx, ny) = get_subpath_start(segments, idx);
                calc_curves_angle(
                    px, py, x2, y2,
                    x, y,
                    nx, ny, nx, ny,
                )
            }
            (_, Segment::ClosePath) => 0.0,
        }
    } else {
        // Middle segments.

        let seg1 = segments[idx];
        let seg2 = segments[idx + 1];

        // Not sure if there is a better way.
        match (seg1, seg2) {
            (Segment::MoveTo { x: mx, y: my }, Segment::LineTo { x, y }) => {
                calc_line_angle(mx, my, x, y)
            }
            (Segment::MoveTo { x: mx, y: my }, Segment::CurveTo { x1, y1, .. }) => {
                calc_line_angle(mx, my, x1, y1)
            }
            (Segment::LineTo { x: x1, y: y1 }, Segment::LineTo { x: x2, y: y2 }) => {
                let (px, py) = get_prev_vertex(segments, idx);
                calc_angle(px, py, x1, y1,
                           x1, y1, x2, y2)
            }
            (Segment::CurveTo { x2: c1_x2, y2: c1_y2, x, y, .. },
                Segment::CurveTo { x1: c2_x1, y1: c2_y1, x: nx, y: ny, .. }) => {
                let (px, py) = get_prev_vertex(segments, idx);
                calc_curves_angle(
                    px, py, c1_x2, c1_y2,
                    x, y,
                    c2_x1, c2_y1, nx, ny,
                )
            }
            (Segment::LineTo { x, y },
                Segment::CurveTo { x1, y1, x: nx, y: ny, .. }) => {
                let (px, py) = get_prev_vertex(segments, idx);
                calc_curves_angle(
                    px, py, px, py,
                    x, y,
                    x1, y1, nx, ny,
                )
            }
            (Segment::CurveTo { x2, y2, x, y, .. },
                Segment::LineTo { x: nx, y: ny }) => {
                let (px, py) = get_prev_vertex(segments, idx);
                calc_curves_angle(
                    px, py, x2, y2,
                    x, y,
                    nx, ny, nx, ny,
                )
            }
            (Segment::LineTo { x, y }, Segment::MoveTo { .. }) => {
                let (px, py) = get_prev_vertex(segments, idx);
                calc_line_angle(px, py, x, y)
            }
            (Segment::CurveTo { x2, y2, x, y, .. }, Segment::MoveTo { .. }) => {
                if x.fuzzy_eq(&x2) && y.fuzzy_eq(&y2) {
                    let (px, py) = get_prev_vertex(segments, idx);
                    calc_line_angle(px, py, x, y)
                } else {
                    calc_line_angle(x2, y2, x, y)
                }
            }
            (Segment::LineTo { x, y }, Segment::ClosePath) => {
                let (px, py) = get_prev_vertex(segments, idx);
                let (nx, ny) = get_subpath_start(segments, idx);
                calc_angle(px, py, x, y,
                           x, y, nx, ny)
            }
            (_, Segment::ClosePath) => {
                let (px, py) = get_prev_vertex(segments, idx);
                let (nx, ny) = get_subpath_start(segments, idx);
                calc_line_angle(px, py, nx, ny)
            }
            (_, Segment::MoveTo { .. }) |
            (Segment::ClosePath, _) => {
                0.0
            }
        }
    }
}

fn calc_line_angle(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
) -> f64 {
    calc_angle(x1, y1, x2, y2, x1, y1, x2, y2)
}

fn calc_curves_angle(
    px: f64,  py: f64,  // previous vertex
    cx1: f64, cy1: f64, // previous control point
    x: f64,   y: f64,   // current vertex
    cx2: f64, cy2: f64, // next control point
    nx: f64,  ny: f64,  // next vertex
) -> f64 {
    if cx1.fuzzy_eq(&x) && cy1.fuzzy_eq(&y) {
        calc_angle(px, py, x, y, x, y, cx2, cy2)
    } else if x.fuzzy_eq(&cx2) && y.fuzzy_eq(&cy2) {
        calc_angle(cx1, cy1, x, y, x, y, nx, ny)
    } else {
        calc_angle(cx1, cy1, x, y, x, y, cx2, cy2)
    }
}

fn calc_angle(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    x3: f64, y3: f64,
    x4: f64, y4: f64,
) -> f64 {
    use std::f64::consts::*;

    fn normalize(rad: f64) -> f64 {
        let v = rad % (PI * 2.0);
        if v < 0.0 { v + PI * 2.0 } else { v }
    }

    fn vector_angle(vx: f64, vy: f64) -> f64 {
        let rad = vy.atan2(vx);
        if rad.is_nan() { 0.0 } else { normalize(rad) }
    }

    let in_a  = vector_angle(x2 - x1, y2 - y1);
    let out_a = vector_angle(x4 - x3, y4 - y3);
    let d = (out_a - in_a) * 0.5;

    let mut angle = in_a + d;
    if FRAC_PI_2 < d.abs() {
        angle -= PI;
    }

    normalize(angle) * 180.0 / PI
}

fn get_subpath_start(segments: &[Segment], idx: usize) -> (f64, f64) {
    let offset = segments.len() - idx;
    for seg in segments.iter().rev().skip(offset) {
        if let Segment::MoveTo { x, y } = *seg {
            return (x, y);
        }
    }

    return (0.0, 0.0)
}

fn get_prev_vertex(segments: &[Segment], idx: usize) -> (f64, f64) {
    match segments[idx - 1] {
        Segment::MoveTo { x, y } => (x, y),
        Segment::LineTo { x, y } => (x, y),
        Segment::CurveTo { x, y, .. } => (x, y),
        Segment::ClosePath => get_subpath_start(segments, idx),
    }
}

fn convert_rect(node: &svgdom::Node, state: &State) -> Rect {
    Rect::new(
        node.convert_user_length(AId::RefX, state, Length::zero()),
        node.convert_user_length(AId::RefY, state, Length::zero()),
        node.convert_user_length(AId::MarkerWidth, state, Length::new_number(3.0)),
        node.convert_user_length(AId::MarkerHeight, state, Length::new_number(3.0)),
    )
}

fn convert_orientation(attrs: &svgdom::Attributes) -> MarkerOrientation {
    match attrs.get_value(AId::Orient) {
        Some(AValue::Angle(angle)) => {
            let a = match angle.unit {
                svgdom::AngleUnit::Degrees  => angle.num,
                svgdom::AngleUnit::Gradians => angle.num * 180.0 / 200.0,
                svgdom::AngleUnit::Radians  => angle.num * 180.0 / f64::consts::PI,
            };

            MarkerOrientation::Angle(a)
        }
        Some(AValue::String(s)) if s == "auto" => {
            MarkerOrientation::Auto
        }
        _ => {
            MarkerOrientation::Angle(0.0)
        }
    }
}
