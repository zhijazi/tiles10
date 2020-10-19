#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical
}

#[derive(Debug, Clone)]
pub enum NodeType<T> {
    Separator(Orientation, Box<Node<T>>, Box<Node<T>>),
    Empty,
    Window(T)
}

// TODO: Arbitrary number of children?
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub node_type: NodeType<T>,
    pub dim: Dimensions,
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

pub fn untile<T: std::fmt::Debug + Copy + PartialEq>(mut root: &mut Node<T>, window_val: &T) {
    match &mut root.node_type {
        NodeType::Window(_) => return,
        NodeType::Empty => return, // for now, assume Empty cannot have children
        NodeType::Separator(_, left_child, right_child) => {
            if let NodeType::Window(child) = &left_child.node_type {
                if child == window_val {
                    // left child is deleted window
                    root.node_type = right_child.node_type.clone();
                    resize_children(root);
                    return;
                }
            } 

            if let NodeType::Window(child) = &right_child.node_type {
                if child == window_val {
                    // right child is deleted window
                    root.node_type = left_child.node_type.clone();
                    resize_children(root);
                    return;
                }
            }

            untile(left_child, &window_val);
            untile(right_child, &window_val);
        }
    }
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
    };

    root.node_type = NodeType::Separator(orientation, Box::new(tmp), Box::new(new_win));
    if let NodeType::Separator(_, left_child, right_child) = &mut root.node_type {
        resize_children(left_child);
        resize_children(right_child);
    }
}

fn resize_children<T: std::clone::Clone>(root: &mut Node<T>) {
    // TODO can do let (left,right) = match root.node_type ... 
    match &mut root.node_type {
        NodeType::Separator(Orientation::Horizontal, left_child, right_child) => {
            let (left, right) = tile_horizontal(&root.dim);
            left_child.dim = left;
            right_child.dim = right;
            resize_children::<T>(left_child);
            resize_children::<T>(right_child);
        },
        NodeType::Separator(Orientation::Vertical, left_child, right_child) => {
            let (left, right) = tile_vertical(&root.dim);
            left_child.dim = left;
            right_child.dim = right;
            resize_children::<T>(left_child);
            resize_children::<T>(right_child);
        }
        _ => return
    }
}
