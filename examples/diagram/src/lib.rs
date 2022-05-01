use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::ops::Bound::{Excluded, Unbounded};

use geo::prelude::BoundingRect;
use geo::GeometryCollection;
use itertools::Itertools;

use crate::geometry::h_v_line_intersection;
use crate::primitives::{HorizontalSegment, Padding, PortNumber, Ports, Unit, VerticalSegment};

pub mod geometry;
pub mod primitives;

enum HorizontalLineEventIteratorState {
    Open,
    LeftPort(PortNumber),
    RightPort(PortNumber),
    Close,
    End,
}

type VerticalPosition = Unit;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum HorizontalLineEventType {
    Open,
    LeftPort(PortNumber),
    RightPort(PortNumber),
    Close,
}

#[derive(Clone, Debug)]
struct HorizontalLineEvent<'a> {
    pub r#type: HorizontalLineEventType,
    pub vertical_position: VerticalPosition,
    pub geom_box: &'a GeomBox,
}

impl<'a> Eq for HorizontalLineEvent<'a> {}

impl<'a> PartialEq<Self> for HorizontalLineEvent<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.vertical_position == other.vertical_position
    }
}

impl<'a> PartialOrd<Self> for HorizontalLineEvent<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.vertical_position.partial_cmp(&other.vertical_position)
    }
}

impl<'a> Ord for HorizontalLineEvent<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.vertical_position.cmp(&other.vertical_position)
    }
}

/// Iterate over y-values of interesting horizontal segments for a GeomBox. Will not be sorted and
/// may contain duplicates.
struct HorizontalLineEventIterator<'a> {
    state: HorizontalLineEventIteratorState,
    geom_box: &'a GeomBox,
    remaining_lines: u32,
}

impl<'a> HorizontalLineEventIterator<'a> {
    pub fn new(geom_box: &'a GeomBox) -> Self {
        let top_lines = 1;
        let left_port_lines = geom_box.ports.left.0;
        let right_port_lines = geom_box.ports.right.0;
        let bottom_lines = 1;
        Self {
            state: HorizontalLineEventIteratorState::Open,
            geom_box,
            remaining_lines: (top_lines + left_port_lines + right_port_lines + bottom_lines + 2) as u32,
        }
    }
}

impl<'a> Iterator for HorizontalLineEventIterator<'a> {
    type Item = HorizontalLineEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            HorizontalLineEventIteratorState::Open => {
                if self.geom_box.ports.left.0 != 0 {
                    self.state = HorizontalLineEventIteratorState::LeftPort(PortNumber(0));
                } else if self.geom_box.ports.right.0 != 0 {
                    self.state = HorizontalLineEventIteratorState::RightPort(PortNumber(0));
                } else {
                    self.state = HorizontalLineEventIteratorState::Close;
                }
                self.remaining_lines -= 1;
                Some(HorizontalLineEvent {
                    r#type: HorizontalLineEventType::Open,
                    vertical_position: self.geom_box.top_y(UsePadding::Yes),
                    geom_box: self.geom_box,
                })
            }
            HorizontalLineEventIteratorState::LeftPort(PortNumber(current)) => {
                if current == self.geom_box.ports.left.0 - 1 {
                    if self.geom_box.ports.right.0 != 0 {
                        self.state = HorizontalLineEventIteratorState::RightPort(PortNumber(0));
                    } else {
                        self.state = HorizontalLineEventIteratorState::Close;
                    }
                } else {
                    self.state = HorizontalLineEventIteratorState::LeftPort(PortNumber(current + 1));
                }
                self.remaining_lines -= 1;
                Some(HorizontalLineEvent {
                    r#type: HorizontalLineEventType::LeftPort(PortNumber(current)),
                    vertical_position: self.geom_box.get_left_port(PortNumber(current), UsePadding::No).y,
                    geom_box: self.geom_box,
                })
            }
            HorizontalLineEventIteratorState::RightPort(PortNumber(current)) => {
                if current == self.geom_box.ports.right.0 - 1 {
                    self.state = HorizontalLineEventIteratorState::Close;
                } else {
                    self.state = HorizontalLineEventIteratorState::RightPort(PortNumber(current + 1));
                }
                self.remaining_lines -= 1;
                Some(HorizontalLineEvent {
                    r#type: HorizontalLineEventType::RightPort(PortNumber(current)),
                    vertical_position: self
                        .geom_box
                        .get_right_port(PortNumber(current), UsePadding::No)
                        .y,
                    geom_box: self.geom_box,
                })
            }
            HorizontalLineEventIteratorState::Close => {
                self.state = HorizontalLineEventIteratorState::End;
                self.remaining_lines -= 1;
                Some(HorizontalLineEvent {
                    r#type: HorizontalLineEventType::Close,
                    vertical_position: self.geom_box.bottom_y(UsePadding::Yes),
                    geom_box: self.geom_box,
                })
            }
            HorizontalLineEventIteratorState::End => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining_lines as usize, Some(self.remaining_lines as usize))
    }
}

impl<'a> ExactSizeIterator for HorizontalLineEventIterator<'a> {}

enum VerticalLineEventIteratorState {
    Open,
    TopPort(PortNumber),
    BottomPort(PortNumber),
    Close,
    End,
}

type HorizontalPosition = Unit;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum VerticalLineEventType {
    Open,
    TopPort(PortNumber),
    BottomPort(PortNumber),
    Close,
}

#[derive(Clone, Debug)]
struct VerticalLineEvent<'a> {
    pub r#type: VerticalLineEventType,
    pub horizontal_position: HorizontalPosition,
    pub geom_box: &'a GeomBox,
}

impl<'a> Eq for VerticalLineEvent<'a> {}

impl<'a> PartialEq<Self> for VerticalLineEvent<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.horizontal_position == other.horizontal_position
    }
}

impl<'a> PartialOrd<Self> for VerticalLineEvent<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.horizontal_position.partial_cmp(&other.horizontal_position)
    }
}

impl<'a> Ord for VerticalLineEvent<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.horizontal_position.cmp(&other.horizontal_position)
    }
}

/// Iterate over x-values of interesting vertical segments for a GeomBox. Will not be sorted and
/// may contain duplicates.
struct VerticalLineEventIterator<'a> {
    state: VerticalLineEventIteratorState,
    geom_box: &'a GeomBox,
    remaining_lines: u32,
}

impl<'a> VerticalLineEventIterator<'a> {
    pub fn new(geom_box: &'a GeomBox) -> Self {
        let left_lines = 1;
        let top_port_lines = geom_box.ports.top.0;
        let bottom_port_lines = geom_box.ports.bottom.0;
        let right_lines = 1;
        Self {
            state: VerticalLineEventIteratorState::Open,
            geom_box,
            remaining_lines: (left_lines + top_port_lines + bottom_port_lines + right_lines) as u32,
        }
    }
}

impl<'a> Iterator for VerticalLineEventIterator<'a> {
    type Item = VerticalLineEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            VerticalLineEventIteratorState::Open => {
                if self.geom_box.ports.top.0 != 0 {
                    self.state = VerticalLineEventIteratorState::TopPort(PortNumber(0));
                } else if self.geom_box.ports.bottom.0 != 0 {
                    self.state = VerticalLineEventIteratorState::BottomPort(PortNumber(0));
                } else {
                    self.state = VerticalLineEventIteratorState::Close;
                }
                self.remaining_lines -= 1;
                Some(VerticalLineEvent {
                    r#type: VerticalLineEventType::Open,
                    horizontal_position: self.geom_box.left_x(UsePadding::Yes),
                    geom_box: self.geom_box,
                })
            }
            VerticalLineEventIteratorState::TopPort(PortNumber(current)) => {
                if current == self.geom_box.ports.top.0 - 1 {
                    if self.geom_box.ports.bottom.0 != 0 {
                        self.state = VerticalLineEventIteratorState::BottomPort(PortNumber(0));
                    } else {
                        self.state = VerticalLineEventIteratorState::Close;
                    }
                } else {
                    self.state = VerticalLineEventIteratorState::TopPort(PortNumber(current + 1));
                }
                self.remaining_lines -= 1;
                Some(VerticalLineEvent {
                    r#type: VerticalLineEventType::TopPort(PortNumber(current)),
                    horizontal_position: self.geom_box.get_top_port(PortNumber(current), UsePadding::No).x,
                    geom_box: self.geom_box,
                })
            }
            VerticalLineEventIteratorState::BottomPort(PortNumber(current)) => {
                if current == self.geom_box.ports.bottom.0 - 1 {
                    self.state = VerticalLineEventIteratorState::Close;
                } else {
                    self.state = VerticalLineEventIteratorState::BottomPort(PortNumber(current + 1));
                }
                self.remaining_lines -= 1;
                Some(VerticalLineEvent {
                    r#type: VerticalLineEventType::BottomPort(PortNumber(current)),
                    horizontal_position: self
                        .geom_box
                        .get_bottom_port(PortNumber(current), UsePadding::No)
                        .x,
                    geom_box: self.geom_box,
                })
            }
            VerticalLineEventIteratorState::Close => {
                self.state = VerticalLineEventIteratorState::End;
                self.remaining_lines -= 1;
                Some(VerticalLineEvent {
                    r#type: VerticalLineEventType::Close,
                    horizontal_position: self.geom_box.right_x(UsePadding::Yes),
                    geom_box: self.geom_box,
                })
            }
            VerticalLineEventIteratorState::End => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining_lines as usize, Some(self.remaining_lines as usize))
    }
}

impl<'a> ExactSizeIterator for VerticalLineEventIterator<'a> {}

/// GeomBox represents a box in 2D. It also comes with
/// - padding (how much space an incoming line must travel straight for into a port) and
/// - ports (additional connectors on sides).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GeomBox {
    pub rect: geo::Rect<Unit>,
    pub padding: Padding,
    pub ports: Ports,
}

#[derive(Clone, Debug)]
struct GeomBoxSortedLeftToRight<'a>(&'a GeomBox);

impl<'a> Eq for GeomBoxSortedLeftToRight<'a> {}

impl<'a> PartialEq<Self> for GeomBoxSortedLeftToRight<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> PartialOrd<Self> for GeomBoxSortedLeftToRight<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for GeomBoxSortedLeftToRight<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        for (first, second) in self
            .0
            .horizontal_sort_amounts()
            .into_iter()
            .zip(other.0.horizontal_sort_amounts())
        {
            let cmp = first.cmp(&second);
            match cmp {
                Ordering::Greater | Ordering::Less => return cmp,
                _ => continue,
            }
        }
        Ordering::Equal
    }
}

#[derive(Clone, Debug)]
struct GeomBoxSortedTopToBottom<'a>(&'a GeomBox);

impl<'a> Eq for GeomBoxSortedTopToBottom<'a> {}

impl<'a> PartialEq<Self> for GeomBoxSortedTopToBottom<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> PartialOrd<Self> for GeomBoxSortedTopToBottom<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for GeomBoxSortedTopToBottom<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        for (first, second) in self
            .0
            .vertical_sort_amounts()
            .into_iter()
            .zip(other.0.vertical_sort_amounts())
        {
            let cmp = first.cmp(&second);
            match cmp {
                Ordering::Greater | Ordering::Less => return cmp,
                _ => continue,
            }
        }
        Ordering::Equal
    }
}

#[derive(Clone, Copy, Debug)]
enum UsePadding {
    Yes,
    No,
}

impl GeomBox {
    fn horizontal_sort_amounts(&self) -> [Unit; 4] {
        [
            self.left_x(UsePadding::Yes),
            self.right_x(UsePadding::Yes),
            self.top_y(UsePadding::Yes),
            self.bottom_y(UsePadding::Yes),
        ]
    }

    fn vertical_sort_amounts(&self) -> [Unit; 4] {
        [
            self.top_y(UsePadding::Yes),
            self.bottom_y(UsePadding::Yes),
            self.left_x(UsePadding::Yes),
            self.right_x(UsePadding::Yes),
        ]
    }

    fn padded_rect(&self) -> geo::Rect<Unit> {
        geo::Rect::new(
            (self.left_x(UsePadding::Yes), self.top_y(UsePadding::Yes)),
            (self.right_x(UsePadding::Yes), self.bottom_y(UsePadding::Yes)),
        )
    }

    fn top_y(&self, use_padding: UsePadding) -> Unit {
        match use_padding {
            UsePadding::Yes => self.rect.min().y - self.padding.top,
            UsePadding::No => self.rect.min().y,
        }
    }

    fn right_x(&self, use_padding: UsePadding) -> Unit {
        match use_padding {
            UsePadding::Yes => self.rect.max().x + self.padding.right,
            UsePadding::No => self.rect.max().x,
        }
    }

    fn bottom_y(&self, use_padding: UsePadding) -> Unit {
        match use_padding {
            UsePadding::Yes => self.rect.max().y + self.padding.bottom,
            UsePadding::No => self.rect.max().y,
        }
    }

    fn left_x(&self, use_padding: UsePadding) -> Unit {
        match use_padding {
            UsePadding::Yes => self.rect.min().x - self.padding.left,
            UsePadding::No => self.rect.min().x,
        }
    }

    fn get_top_port(&self, port_number: PortNumber, use_padding: UsePadding) -> geo::Coordinate<Unit> {
        let x: Unit = self.left_x(UsePadding::No);
        let dx: Unit =
            self.rect.height() * (Unit::from(port_number.0 + 1) / Unit::from(self.ports.top.0 + 1));
        geo::Coordinate::from((x + dx, self.top_y(use_padding)))
    }

    fn get_right_port(&self, port_number: PortNumber, use_padding: UsePadding) -> geo::Coordinate<Unit> {
        let y: Unit = self.top_y(UsePadding::No);
        let dy: Unit =
            self.rect.width() * (Unit::from(port_number.0 + 1) / Unit::from(self.ports.right.0 + 1));
        geo::Coordinate::from((self.right_x(use_padding), y + dy))
    }

    fn get_bottom_port(&self, port_number: PortNumber, use_padding: UsePadding) -> geo::Coordinate<Unit> {
        let x: Unit = self.left_x(UsePadding::No);
        let dx: Unit =
            self.rect.height() * (Unit::from(port_number.0 + 1) / Unit::from(self.ports.bottom.0 + 1));
        geo::Coordinate::from((x + dx, self.bottom_y(use_padding)))
    }

    fn get_left_port(&self, port_number: PortNumber, use_padding: UsePadding) -> geo::Coordinate<Unit> {
        let y: Unit = self.top_y(UsePadding::No);
        let dy: Unit =
            self.rect.width() * (Unit::from(port_number.0 + 1) / Unit::from(self.ports.left.0 + 1));
        geo::Coordinate::from((self.left_x(use_padding), y + dy))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Diagram {
    pub boxes: Vec<GeomBox>,
    pub bounding_box: geo::Rect<Unit>,
}

impl Diagram {
    pub fn new(boxes: Vec<GeomBox>) -> Self {
        let bounding_box: geo::Rect<Unit> = GeometryCollection(
            boxes
                .iter()
                .map(|geom_box| geom_box.padded_rect())
                .map(geo::Geometry::Rect)
                .collect(),
        )
        .bounding_rect()
        .unwrap();

        Self { boxes, bounding_box }
    }
}

/// We generate the non-overlap constraints in each dimension in O(|V | log |V |) time using a
/// line-sweep algorithm related to standard rectangle overlap detection methods [12]. First, consider
/// the generation of horizontal constraints. We use a vertical sweep through the nodes, keeping a
/// horizontal “scan line” list of open nodes with each node having references to its closest left and
/// right neighbors (or more exactly the neighbors with which it is currently necessary to generate a
/// non-overlap constraint). When the scan line reaches the top of a new node, this is added to the list
/// and its neighbors computed. When the bottom of a node is reached the the separation constraints for
/// the node are generated and the node is removed from the list.
///
/// Fast Node Overlap Removal - Tim Dwyer and Kim Marriott and Peter James Stuckey - 2005
///
/// The interesting horizontal segments can be generated in O(n log n) time where n is the number of
/// objects in the diagram by using a variant of the line- sweep algorithm from [3,4]. This uses a
/// vertical sweep through the objects in the diagram, keeping a horizontal “scan line” list of open
/// objects with each node having references to its closest left and right neighbors. Interesting,
/// horizontal segments are generated, when an object is opened, closed, or a connection point is
/// reached. Dually, the interesting vertical segments can generated in O(n log n) time by using a
/// horizontal sweep. The last step takes O(n^2) time since there are O(n) interesting horizontal
/// and vertical segments.
///
/// Orthogonal connector routing - Wybrow, Michael and Marriott, Kim and Stuckey, Peter J - 2009
/// page 4
pub fn get_interesting_horizontal_segments(diagram: &Diagram) -> Vec<HorizontalSegment> {
    let geom_boxes = &diagram.boxes;
    let diagram_min_x = diagram.bounding_box.min().x;
    let diagram_max_x = diagram.bounding_box.max().x;
    let mut open_geom_boxes: BTreeSet<GeomBoxSortedLeftToRight> = BTreeSet::new();
    let horizontal_line_events: Vec<HorizontalLineEvent> = geom_boxes
        .iter()
        .flat_map(HorizontalLineEventIterator::new)
        .sorted_unstable_by_key(|horizontal_line_event| horizontal_line_event.vertical_position)
        .collect();
    let mut result: Vec<_> = Vec::with_capacity(horizontal_line_events.len());
    for event in horizontal_line_events {
        let y = event.vertical_position;
        let left_x = match &event.r#type {
            HorizontalLineEventType::RightPort(_port_number) => event.geom_box.right_x(UsePadding::No),
            _ => {
                let maybe_left_geom_box = open_geom_boxes
                    .range((Unbounded, Excluded(GeomBoxSortedLeftToRight(event.geom_box))))
                    .next_back();
                match maybe_left_geom_box {
                    None => diagram_min_x,
                    Some(GeomBoxSortedLeftToRight(geom_box)) => geom_box.right_x(UsePadding::Yes),
                }
            }
        };
        let right_x = match &event.r#type {
            HorizontalLineEventType::LeftPort(_port_number) => event.geom_box.left_x(UsePadding::No),
            _ => {
                let maybe_right_geom_box = open_geom_boxes
                    .range((Excluded(GeomBoxSortedLeftToRight(event.geom_box)), Unbounded))
                    .next();
                match maybe_right_geom_box {
                    None => diagram_max_x,
                    Some(GeomBoxSortedLeftToRight(geom_box)) => geom_box.left_x(UsePadding::Yes),
                }
            }
        };
        let new_line: geo::Line<Unit> = geo::Line::new((left_x, y), (right_x, y));
        result.push(new_line.into());

        match event.r#type {
            HorizontalLineEventType::Open => {
                open_geom_boxes.insert(GeomBoxSortedLeftToRight(event.geom_box));
            }
            HorizontalLineEventType::Close => {
                open_geom_boxes.remove(&GeomBoxSortedLeftToRight(event.geom_box));
            }
            _ => {}
        }
    }
    result
}

pub fn get_interesting_vertical_segments(diagram: &Diagram) -> Vec<VerticalSegment> {
    let geom_boxes = &diagram.boxes;
    let diagram_min_y = diagram.bounding_box.min().y;
    let diagram_max_y = diagram.bounding_box.max().y;
    let mut open_geom_boxes: BTreeSet<GeomBoxSortedTopToBottom> = BTreeSet::new();
    let vertical_line_events: Vec<VerticalLineEvent> = geom_boxes
        .iter()
        .flat_map(VerticalLineEventIterator::new)
        .sorted_unstable_by_key(|vertical_line_event| vertical_line_event.horizontal_position)
        .collect();
    for vle in &vertical_line_events {
        println!("vertical_line_event: {:?}", vle);
        println!("---");
    }
    let mut result: Vec<_> = Vec::with_capacity(vertical_line_events.len());
    for event in vertical_line_events {
        let x = event.horizontal_position;
        let top_y = match &event.r#type {
            VerticalLineEventType::BottomPort(_port_number) => event.geom_box.bottom_y(UsePadding::No),
            _ => {
                let maybe_top_geom_box = open_geom_boxes
                    .range((Unbounded, Excluded(GeomBoxSortedTopToBottom(event.geom_box))))
                    .next_back();
                match maybe_top_geom_box {
                    None => diagram_min_y,
                    Some(GeomBoxSortedTopToBottom(geom_box)) => geom_box.bottom_y(UsePadding::Yes),
                }
            }
        };
        let bottom_y = match &event.r#type {
            VerticalLineEventType::TopPort(_port_number) => event.geom_box.top_y(UsePadding::No),
            _ => {
                let maybe_bottom_geom_box = open_geom_boxes
                    .range((Excluded(GeomBoxSortedTopToBottom(event.geom_box)), Unbounded))
                    .next();
                match maybe_bottom_geom_box {
                    None => diagram_max_y,
                    Some(GeomBoxSortedTopToBottom(geom_box)) => geom_box.top_y(UsePadding::Yes),
                }
            }
        };

        let new_line: geo::Line<Unit> = geo::Line::new((x, top_y), (x, bottom_y));
        result.push(new_line.into());

        match event.r#type {
            VerticalLineEventType::Open => {
                open_geom_boxes.insert(GeomBoxSortedTopToBottom(event.geom_box));
            }
            VerticalLineEventType::Close => {
                open_geom_boxes.remove(&GeomBoxSortedTopToBottom(event.geom_box));
            }
            _ => {}
        }
    }
    result
}

#[derive(Debug)]
pub struct OrthogonalVisibilityGraph {
    pub interesting_horizontal_segments: HashSet<HorizontalSegment, fasthash::sea::Hash64>,
    pub interesting_vertical_segments: HashSet<VerticalSegment, fasthash::sea::Hash64>,
    pub vertices: HashSet<geo::Coordinate<Unit>, fasthash::sea::Hash64>,
    pub edges: HashSet<geo::Line<Unit>, fasthash::sea::Hash64>,
}

impl OrthogonalVisibilityGraph {
    pub fn new(diagram: &Diagram) -> OrthogonalVisibilityGraph {
        let interesting_horizontal_segments = get_interesting_horizontal_segments(diagram);
        let mut interesting_horizontal_segments_lookup =
            HashSet::with_capacity_and_hasher(interesting_horizontal_segments.len(), fasthash::sea::Hash64);
        interesting_horizontal_segments_lookup.extend(interesting_horizontal_segments.into_iter());

        let interesting_vertical_segments = get_interesting_vertical_segments(diagram);
        let mut interesting_vertical_segments_lookup =
            HashSet::with_capacity_and_hasher(interesting_vertical_segments.len(), fasthash::sea::Hash64);
        interesting_vertical_segments_lookup.extend(interesting_vertical_segments.into_iter());

        let mut vertices: HashSet<geo::Coordinate<Unit>, fasthash::sea::Hash64> =
            HashSet::with_capacity_and_hasher(
                interesting_horizontal_segments_lookup.len() * interesting_vertical_segments_lookup.len(),
                fasthash::sea::Hash64,
            );
        for geom_box in &diagram.boxes {
            for i in 0..geom_box.ports.top.0 {
                vertices.insert(geom_box.get_top_port(PortNumber(i), UsePadding::No));
            }
            for i in 0..geom_box.ports.right.0 {
                vertices.insert(geom_box.get_right_port(PortNumber(i), UsePadding::No));
            }
            for i in 0..geom_box.ports.bottom.0 {
                vertices.insert(geom_box.get_bottom_port(PortNumber(i), UsePadding::No));
            }
            for i in 0..geom_box.ports.left.0 {
                vertices.insert(geom_box.get_left_port(PortNumber(i), UsePadding::No));
            }
        }

        // TODO replace O(n^2) with a sweep
        interesting_horizontal_segments_lookup.iter().for_each(|h| {
            interesting_vertical_segments_lookup
                .iter()
                .for_each(|v| match h_v_line_intersection(*h, *v) {
                    None => {}
                    Some(geo::Coordinate { x, y }) => {
                        vertices.insert([x, y].into());
                    }
                })
        });

        let mut edges =
            HashSet::with_capacity_and_hasher(vertices.len() * vertices.len(), fasthash::sea::Hash64);

        // TODO replace O(n^2) either with another sweep or at the same time as intersection calculation
        for v1 in &vertices {
            for v2 in &vertices {
                if v1.x == v2.x && v1.y <= v2.y {
                    if interesting_vertical_segments_lookup
                        .contains(&VerticalSegment(geo::Line::new((v1.x, v1.y), (v2.x, v2.y))))
                        || interesting_vertical_segments_lookup
                            .contains(&VerticalSegment(geo::Line::new((v2.x, v2.y), (v1.x, v1.y))))
                    {
                        edges.insert(geo::Line::new(*v1, *v2));
                    }
                } else if v1.y == v2.y && v1.x <= v2.x {
                    if interesting_horizontal_segments_lookup
                        .contains(&HorizontalSegment(geo::Line::new((v1.x, v1.y), (v2.x, v2.y))))
                        || interesting_horizontal_segments_lookup
                            .contains(&HorizontalSegment(geo::Line::new((v2.x, v2.y), (v1.x, v1.y))))
                    {
                        edges.insert(geo::Line::new(*v1, *v2));
                    }
                }
            }
        }

        Self {
            interesting_horizontal_segments: interesting_horizontal_segments_lookup,
            interesting_vertical_segments: interesting_vertical_segments_lookup,
            vertices,
            edges,
        }
    }
}

pub fn new_rect<T>(first: (T, T), second: (T, T)) -> geo::Rect<Unit>
where
    T: std::fmt::Debug + Into<Unit>,
{
    geo::Rect::new(
        geo::Coordinate {
            x: first.0.into(),
            y: first.1.into(),
        },
        geo::Coordinate {
            x: second.0.into(),
            y: second.1.into(),
        },
    )
}

pub fn new_line<T>(first: (T, T), second: (T, T)) -> geo::Line<Unit>
where
    T: Into<Unit> + std::fmt::Debug,
{
    geo::Line::from([
        (first.0.into(), first.1.into()),
        (second.0.into(), second.1.into()),
    ])
}

fn line_to_string(line: Vec<impl Into<geo::Line<Unit>> + Clone>) -> String {
    line.into_iter()
        .map(|s| {
            let line: geo::Line<Unit> = s.clone().into();
            format!(
                "{{({},{}),({},{})}}",
                line.start.x, line.start.y, line.end.x, line.end.y
            )
        })
        .join(", ")
}

fn points_to_string(line: &Vec<geo::Coordinate<Unit>>) -> String {
    line.iter().map(|s| format!("({},{})", s.x, s.y)).join(", ")
}

#[cfg(test)]
mod diagram_geom_tests {
    use approx::assert_abs_diff_eq;
    use num_traits::ToPrimitive;
    use proptest::prelude::*;

    use super::*;

    #[test]
    pub fn horizontal_line_y_iterator_example_01() {
        // === given ===
        let geom_box = GeomBox {
            rect: new_rect((10.0, 10.0), (20.0, 20.0)),
            padding: Padding::new_uniform(0.0),
            ports: Ports::new(1u8, 2u8, 3u8, 4u8),
        };

        // === when ===
        let y_iterator = HorizontalLineEventIterator::new(&geom_box);

        // === then ===
        let mut ys: Vec<HorizontalLineEvent> = y_iterator.sorted_unstable().collect();
        ys.dedup();
        let ys_expected = vec![
            10.0,
            12.0,
            13.0 + 1.0 / 3.0,
            14.0,
            16.0,
            16.0 + 2.0 / 3.0,
            18.0,
            20.0,
        ];
        assert_eq!(ys.len(), ys_expected.len());
        for (expected, actual) in ys_expected.iter().zip(ys) {
            let actual_val = actual.vertical_position.to_f64().unwrap();
            let actual_val = (actual_val * 1000000.0).round() / 1000000.0;
            assert_abs_diff_eq!(actual_val, expected, epsilon = 1e-6);
        }

        for i in 0..=ys_expected.len() {
            // === when ===
            let mut y_iterator = HorizontalLineEventIterator::new(&geom_box);
            for _j in 0..i {
                y_iterator.next();
            }

            // === then ===
            let expected_size = ys_expected.len() - i;
            assert_eq!(y_iterator.size_hint(), (expected_size, Some(expected_size)));
            assert_eq!(y_iterator.len(), expected_size);
        }
    }

    proptest! {
        #[test]
        fn horizontal_y_iterator_doesnt_crash(x1 in 0.0f64..100.0f64,
                                              y1 in 0.0f64..100.0f64,
                                              x2 in 0.0f64..100.0f64,
                                              y2 in 0.0f64..100.0f64,
                                              padding in 0.0f64..100.0f64,
                                              top_port in 0u8..255u8,
                                              right_port in 0u8..255u8,
                                              bottom_port in 0u8..255u8,
                                              left_port in 0u8..255u8) {
            // === given ===
            let geom_box = GeomBox {
                rect: new_rect((x1, y1), (x2, y2)),
                padding: Padding::new_uniform(padding),
                ports: Ports::new(top_port, right_port, bottom_port, left_port),
            };

            // === when ===
            let y_iterator = HorizontalLineEventIterator::new(&geom_box);

            // === then ===
            let mut _ys: Vec<HorizontalLineEvent> = y_iterator.sorted_unstable().collect();
        }
    }

    #[test]
    pub fn get_interesting_horizontal_segments_example_01() {
        // === given ===
        let diagram = Diagram::new(vec![
            GeomBox {
                rect: new_rect((100.0, 100.0), (200.0, 200.0)),
                padding: Padding::new_uniform(10.0),
                ports: Ports::new(1, 1, 0, 0),
            },
            GeomBox {
                rect: new_rect((300.0, 100.0), (400.0, 200.0)),
                padding: Padding::new_uniform(10.0),
                ports: Ports::new(0, 0, 0, 1),
            },
        ]);

        // === when ===
        let segments = super::get_interesting_horizontal_segments(&diagram);

        // === then ===
        assert_eq!(
            segments.as_slice(),
            &[
                // Top line across whole diagram, caused by first box
                HorizontalSegment(new_line((90.0, 90.0), (410.0, 90.0))),
                // Another top line across diagram caused by second box, but truncated by first box.
                HorizontalSegment(new_line((210.0, 90.0), (410.0, 90.0))),
                // Right-port of first box to left padded side of second box
                HorizontalSegment(new_line((200.0, 150.0), (290.0, 150.0))),
                // Left-port of second box to the right padded side of the first box
                HorizontalSegment(new_line((210.0, 150.0), (300.0, 150.0))),
                // Bottom line across diagram caused by first box, but truncated by second box.
                HorizontalSegment(new_line((90.0, 210.0), (290.0, 210.0))),
                // Bottom line across whole diagram caused by second box.
                HorizontalSegment(new_line((90.0, 210.0), (410.0, 210.0))),
            ],
        );
    }

    #[test]
    pub fn get_interesting_vertical_segments_example_01() {
        // === given ===
        let diagram = Diagram::new(vec![
            GeomBox {
                rect: new_rect((100.0, 100.0), (200.0, 200.0)),
                padding: Padding::new_uniform(10.0),
                ports: Ports::new(1u8, 1u8, 0u8, 0u8),
            },
            GeomBox {
                rect: new_rect((300.0, 100.0), (400.0, 200.0)),
                padding: Padding::new_uniform(10.0),
                ports: Ports::new(0u8, 0u8, 0u8, 1u8),
            },
        ]);

        // === when ===
        let segments = super::get_interesting_vertical_segments(&diagram);

        // === then ===
        println!(
            "actual: {:?}",
            line_to_string(segments.iter().map(|s| s.0).collect())
        );
        assert_eq!(
            segments.as_slice(),
            &[
                // Top-to-bottom line caused by first (left-most) box
                VerticalSegment(new_line((90.0, 90.0), (90.0, 210.0))),
                // Line into the first box's top port
                VerticalSegment(new_line((150.0, 90.0), (150.0, 100.0))),
                // Right line caused by first box top to bottom
                VerticalSegment(new_line((210.0, 90.0), (210.0, 210.0))),
                // Top-to-bottom line on left side of second box
                VerticalSegment(new_line((290.0, 90.0), (290.0, 210.0))),
                // Top-to-bottom line on right side of second (right-most) box
                VerticalSegment(new_line((410.0, 90.0), (410.0, 210.0))),
            ],
        );
    }

    #[test]
    pub fn get_orthogonal_visibility_graph_01() {
        // === given ===
        let diagram = Diagram::new(vec![
            GeomBox {
                rect: new_rect((100.0, 100.0), (200.0, 200.0)),
                padding: Padding::new_uniform(10.0),
                ports: Ports::new(1u8, 1u8, 0u8, 0u8),
            },
            GeomBox {
                rect: new_rect((300.0, 100.0), (400.0, 200.0)),
                padding: Padding::new_uniform(10.0),
                ports: Ports::new(0u8, 0u8, 0u8, 1u8),
            },
        ]);

        // === when ===
        let graph = OrthogonalVisibilityGraph::new(&diagram);
        let points = graph.vertices.into_iter().collect();
        let edges: Vec<&geo::Line<Unit>> = graph.edges.iter().collect();

        // === then ===
        println!("points: {:?}", points_to_string(&points));
        println!("edges: {:?}", edges);
        // assert_eq!(points, vec![]);
    }
}
