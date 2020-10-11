#[derive(Clone)]
pub enum Orientation {
    Horizontal,
    Vertical
}

#[derive(Clone)]
pub enum NodeType<T> {
    Separator(Orientation),
    Window(T)
}

#[derive(Clone)]
pub struct Node<T: std::clone::Clone> {
    pub node_type: NodeType<T>,
    pub dim: Dimensions,
    pub left: Box<Option<Node<T>>>,
    pub right: Box<Option<Node<T>>>
}

#[derive(Clone)]
pub struct Dimensions {
    pub x: (i32, i32),
    pub y: (i32, i32)
}

pub fn tile_vertical(dim: &Dimensions) -> (Dimensions, Dimensions) {
    let left_dim = Dimensions {
        x: (dim.x.0, dim.x.1 / 2),
        y: (dim.y.0, dim.y.1)
    };
    let right_dim = Dimensions {
        x: (dim.x.0 + dim.x.1 / 2, dim.x.1 / 2),
        y: (dim.y.0, dim.y.1)
    };
    (left_dim, right_dim)
}

pub fn tile_horizontal(dim: &Dimensions) -> (Dimensions, Dimensions) {
    let top_dim = Dimensions {
        x: (dim.x.0, dim.x.1),
        y: (dim.y.0, dim.y.1 / 2)
    };
    let bottom_dim = Dimensions {
        x: (dim.x.0, dim.x.1),
        y: (dim.y.0 + dim.y.1 / 2, dim.y.1 / 2)
    };
    (top_dim, bottom_dim)
}

pub fn tile<T: std::clone::Clone>(root: &mut Node<T>, orientation: Orientation, new_window: T) {
    let (left_dim, right_dim) = match &orientation {
        Orientation::Horizontal => tile_horizontal(&root.dim),
        Orientation::Vertical => tile_vertical(&root.dim)
    };

    // clone current and set it to left
    let mut tmp = root.clone();
    tmp.dim = left_dim;

    let new_win = Node {
        node_type: NodeType::Window(new_window),
        dim: right_dim,
        left: Box::new(None),
        right: Box::new(None)
    };

    root.node_type = NodeType::Separator(orientation);
    root.left = Box::new(Some(tmp));
    root.right = Box::new(Some(new_win));
}

