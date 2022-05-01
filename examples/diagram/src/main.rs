use diagram::primitives::{Padding, Ports, Unit};
use diagram::{new_rect, Diagram, GeomBox, OrthogonalVisibilityGraph};
use num_traits::ToPrimitive;
use std::rc::Rc;
use usvg::NodeExt;

struct DiagramSpecification {}

struct DiagramSolution {}

fn draw_lines(
    lines: Vec<geo::Line<Unit>>,
    paint: usvg::Paint,
    opacity: usvg::Opacity,
    stroke_width: usvg::StrokeWidth,
) -> Vec<usvg::Path> {
    let mut result = Vec::with_capacity(lines.len());
    for line in lines {
        let mut path_data = usvg::PathData::new();
        path_data.push_move_to(line.start.x.to_f64().unwrap(), line.start.y.to_f64().unwrap());
        path_data.push_line_to(line.end.x.to_f64().unwrap(), line.end.y.to_f64().unwrap());
        let fill = Some(usvg::Fill {
            paint: paint.clone(),
            opacity: opacity.clone(),
            ..usvg::Fill::default()
        });
        let stroke = Some(usvg::Stroke {
            paint: paint.clone(),
            opacity: opacity.clone(),
            width: stroke_width.clone(),
            ..usvg::Stroke::default()
        });
        let path = usvg::Path {
            fill,
            stroke: stroke.clone(),
            data: Rc::new(path_data),
            ..usvg::Path::default()
        };
        result.push(path);
    }
    result
}

fn draw(diagram: Diagram, ovg: OrthogonalVisibilityGraph) {
    let padding = 20.0;
    let size = usvg::Size::new(
        diagram.bounding_box.max().x.to_f64().unwrap() + padding,
        diagram.bounding_box.max().y.to_f64().unwrap() + padding,
    )
    .unwrap();
    let mut rtree = usvg::Tree::create(usvg::Svg {
        size,
        view_box: usvg::ViewBox {
            rect: size.to_rect(0.0, 0.0),
            aspect: usvg::AspectRatio::default(),
        },
    });
    rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
        fill: Some(usvg::Fill {
            paint: usvg::Paint::Color(usvg::Color::white()),
            opacity: usvg::Opacity::new(1.0),
            ..usvg::Fill::default()
        }),
        stroke: None,
        data: Rc::new(usvg::PathData::from_rect(
            usvg::Rect::new(0.0, 0.0, size.width(), size.height()).unwrap(),
        )),
        ..usvg::Path::default()
    }));
    let fill = Some(usvg::Fill {
        paint: usvg::Paint::Color(usvg::Color::white()),
        opacity: usvg::Opacity::new(0.0),
        ..usvg::Fill::default()
    });
    let geom_box_stroke = Some(usvg::Stroke {
        paint: usvg::Paint::Color(usvg::Color::black()),
        opacity: usvg::Opacity::new(1.0),
        ..usvg::Stroke::default()
    });
    for geom_box in &diagram.boxes {
        let rect = usvg::Rect::new(
            geom_box.rect.min().x.to_f64().unwrap(),
            geom_box.rect.min().y.to_f64().unwrap(),
            geom_box.rect.width().to_f64().unwrap(),
            geom_box.rect.height().to_f64().unwrap(),
        )
        .unwrap();
        rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
            fill: fill.clone(),
            stroke: geom_box_stroke.clone(),
            data: Rc::new(usvg::PathData::from_rect(rect)),
            ..usvg::Path::default()
        }));
    }

    let vertex_fill = Some(usvg::Fill {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(0, 0, 255)),
        opacity: usvg::Opacity::new(1.0),
        ..usvg::Fill::default()
    });
    let vertex_stroke = Some(usvg::Stroke {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(0, 0, 255)),
        opacity: usvg::Opacity::new(1.0),
        ..usvg::Stroke::default()
    });
    for vertex in &ovg.vertices {
        let size = 2.0;
        let rect = usvg::Rect::new(
            vertex.x.to_f64().unwrap() - size,
            vertex.y.to_f64().unwrap() - size,
            size * 2.0,
            size * 2.0,
        )
        .unwrap();
        rtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
            fill: vertex_fill.clone(),
            stroke: vertex_stroke.clone(),
            data: Rc::new(usvg::PathData::from_rect(rect)),
            ..usvg::Path::default()
        }));
    }

    let h_lines: Vec<usvg::Path> = draw_lines(
        ovg.interesting_horizontal_segments.iter().map(|h| h.0).collect(),
        usvg::Paint::Color(usvg::Color::new_rgb(255, 0, 0)),
        usvg::Opacity::new(0.0),
        usvg::StrokeWidth::new(3.0),
    );
    let v_lines: Vec<usvg::Path> = draw_lines(
        ovg.interesting_vertical_segments.iter().map(|v| v.0).collect(),
        usvg::Paint::Color(usvg::Color::new_rgb(0, 255, 0)),
        usvg::Opacity::new(0.0),
        usvg::StrokeWidth::new(3.0),
    );
    let edges: Vec<usvg::Path> = draw_lines(
        ovg.edges.into_iter().collect::<Vec<geo::Line<Unit>>>(),
        usvg::Paint::Color(usvg::Color::new_rgb(0, 255, 0)),
        usvg::Opacity::new(0.5),
        usvg::StrokeWidth::new(1.0),
    );
    for line in itertools::chain!(h_lines, v_lines, edges).into_iter() {
        rtree.root().append_kind(usvg::NodeKind::Path(line));
    }

    println!("{}", rtree.to_string(&usvg::XmlOptions::default()));
    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &rtree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap.save_png("/tmp/out.png").unwrap();
}

fn main() {
    let mut geom_boxes = vec![];
    let size = 3;
    for i in 0..size {
        for j in 0..size {
            let x_min = 100.0 + j as f64 * 300.0;
            let x_max = x_min + 100.0;
            let y_min = 100.0 + ((i + 1) / 2) as f64 * 300.0;
            let y_max = y_min + 100.0;
            let ports = match j {
                0 => Ports::new(1, 2, 1, 0),
                _ => Ports::new(1, 0, 1, 2),
            };
            let geom_box = GeomBox {
                rect: new_rect((x_min, y_min), (x_max, y_max)),
                padding: Padding::new_uniform(20.0),
                ports,
            };
            geom_boxes.push(geom_box);
        }
    }
    let diagram = Diagram::new(geom_boxes);
    // println!("diagram: {:?}", &diagram);
    let ovg = OrthogonalVisibilityGraph::new(&diagram);
    // println!("ovg {:?}", &ovg);

    println!(
        "bounding ({:.2}, {:.2}), ({:.2}, {:.2})",
        &diagram.bounding_box.min().x.to_f64().unwrap(),
        &diagram.bounding_box.min().y.to_f64().unwrap(),
        &diagram.bounding_box.max().x.to_f64().unwrap(),
        &diagram.bounding_box.max().y.to_f64().unwrap()
    );
    for geom_box in &diagram.boxes {
        println!(
            "box ({:.2}, {:.2}), ({:.2}, {:.2})",
            geom_box.rect.min().x.to_f64().unwrap(),
            geom_box.rect.min().y.to_f64().unwrap(),
            geom_box.rect.max().x.to_f64().unwrap(),
            geom_box.rect.max().y.to_f64().unwrap()
        );
    }
    for h in &ovg.interesting_horizontal_segments {
        println!(
            "interesting horizontal segment ({:.2}, {:.2}), ({:.2}, {:.2})",
            h.0.start.x.to_f64().unwrap(),
            h.0.start.y.to_f64().unwrap(),
            h.0.end.x.to_f64().unwrap(),
            h.0.end.y.to_f64().unwrap()
        );
    }
    for v in &ovg.interesting_vertical_segments {
        println!(
            "interesting vertical segment ({:.2}, {:.2}), ({:.2}, {:.2})",
            v.0.start.x.to_f64().unwrap(),
            v.0.start.y.to_f64().unwrap(),
            v.0.end.x.to_f64().unwrap(),
            v.0.end.y.to_f64().unwrap()
        );
    }
    for vertex in &ovg.vertices {
        println!(
            "vertex ({:.2}, {:.2})",
            vertex.x.to_f64().unwrap(),
            vertex.y.to_f64().unwrap()
        );
    }
    for edge in &ovg.edges {
        println!(
            "edge ({:.2}, {:.2}), ({:.2}, {:.2})",
            edge.start.x.to_f64().unwrap(),
            edge.start.y.to_f64().unwrap(),
            edge.end.x.to_f64().unwrap(),
            edge.end.y.to_f64().unwrap()
        );
    }
    draw(diagram, ovg);
    println!("** done");
}
