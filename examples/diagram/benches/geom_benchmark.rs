use criterion::{black_box, criterion_group, criterion_main, Criterion};

use diagram::geom::{new_rect, Diagram, GeomBox, OrthogonalVisibilityGraph, Padding, Ports};

fn get_interesting_points_fifty_horizontal_boxes(c: &mut Criterion) {
    let mut geom_boxes = vec![];
    for i in 0..50 {
        let geom_box = GeomBox {
            rect: new_rect(
                (i as f64 * 100.0, i as f64 * 100.0),
                ((i as f64 + 1.0) * 100.0, (i as f64 + 1.0) * 100.0),
            ),
            padding: Padding::new_uniform(10.0),
            ports: Ports::new(1, 1, 1, 1),
        };
        geom_boxes.push(geom_box);
    }
    let diagram = Diagram::new(geom_boxes);

    c.bench_function("Get orthogonal visibility graph - fifty horizontal boxes", |b| {
        b.iter(|| black_box(OrthogonalVisibilityGraph::new(&diagram)));
    });
}

criterion_group!(benches, get_interesting_points_fifty_horizontal_boxes);
criterion_main!(benches);
