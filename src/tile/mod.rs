#[derive(Debug, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical
}

#[derive(Debug, Clone)]
pub enum NodeType<T> {
    Separator(Orientation),
    Empty,
    Window(T)
}

// TODO: Arbitrary number of children?
#[derive(Debug, Clone)]
pub struct Node<T: std::clone::Clone> {
    pub node_type: NodeType<T>,
    pub dim: Dimensions,
    pub left: Option<Box<Node<T>>>,
    pub right: Option<Box<Node<T>>>
}

#[derive(Debug, Clone)]
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
    if let NodeType::Empty =  root.node_type {
        root.node_type = NodeType::Window(new_window);
        return;
    }

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
        left: None,
        right: None
    };

    root.node_type = NodeType::Separator(orientation);
    root.left = Some(Box::new(tmp));
    root.right = Some(Box::new(new_win));

    if let Some(left_child) = &mut root.left {
        resize_children(left_child);
    }

    if let Some(right_child) = &mut root.right {
        resize_children(right_child);
    }
}

fn resize_children<T: std::clone::Clone>(root: &mut Node<T>) {
    match root.node_type {
        NodeType::Separator(Orientation::Horizontal) => {
            let (left, right) = tile_horizontal(&root.dim);

            if let Some(left_win) = &mut root.left {
                left_win.dim = left;
            }
            if let Some(right_win) = &mut root.right {
                right_win.dim = right;
            }
        },
        NodeType::Separator(Orientation::Vertical) => {
            let (left, right) = tile_vertical(&root.dim);
            if let Some(left_win) = &mut root.left {
                left_win.dim = left;
            }

            if let Some(right_win) = &mut root.right {
                right_win.dim = right;
            }
        }
        _ => return
    }

    if let Some(left_node) = &mut root.left {
          resize_children::<T>(left_node);
    }

    if let Some(right_node) = &mut root.right {
          resize_children::<T>(right_node);
    }
}
