#[derive(Clone, PartialEq, Eq, Debug)]
struct GridSizeConstraint {
    grid_size_x: u64,
    grid_size_y: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct DiagramSizeConstraint {
    max_x: u64,
    max_y: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct BoxPaddingConstraint {
    padding: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum DiagramConstraint {
    GridSizeConstraint(GridSizeConstraint),
    DiagramSizeConstraint(DiagramSizeConstraint),
    BoxPaddingConstraint(BoxPaddingConstraint),
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct BoxVariable {
    id: u64,
    rect: euclid::default::Rect<u64>,
}


#[derive(Clone, PartialEq, Eq, Debug)]
enum DiagramVariable {
    BoxVariable(BoxVariable),
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct DiagramSolution {
    variables: Vec<DiagramVariable>,
}

fn main() {
    println!("Hello, world!");
}
