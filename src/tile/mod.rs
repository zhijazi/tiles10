#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum NodeType<T> {
    Separator(Orientation, Box<Node<T>>, Box<Node<T>>),
    Empty,
    Window(T),
}

// TODO: Arbitrary number of children?
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub node_type: NodeType<T>,
    pub dim: Dimensions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dimensions {
    pub x: (i32, i32),
    pub y: (i32, i32),
}

pub fn tile_vertical(dim: &Dimensions) -> (Dimensions, Dimensions) {
    let left_dim = Dimensions {
        x: (dim.x.0, dim.x.1 / 2),
        y: (dim.y.0, dim.y.1),
    };
    let right_dim = Dimensions {
        x: (dim.x.0 + dim.x.1 / 2 + 1, dim.x.1 / 2),
        y: (dim.y.0, dim.y.1),
    };
    (left_dim, right_dim)
}

pub fn tile_horizontal(dim: &Dimensions) -> (Dimensions, Dimensions) {
    let top_dim = Dimensions {
        x: (dim.x.0, dim.x.1),
        y: (dim.y.0, dim.y.1 / 2),
    };
    let bottom_dim = Dimensions {
        x: (dim.x.0, dim.x.1),
        y: (dim.y.0 + dim.y.1 / 2 + 1, dim.y.1 / 2),
    };
    (top_dim, bottom_dim)
}

pub fn untile<T: Copy + PartialEq>(mut root: &mut Node<T>, window_val: &T) {
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

pub fn tile<T: Clone>(root: &mut Node<T>, orientation: Orientation, new_window: T) {
    if let NodeType::Empty = root.node_type {
        root.node_type = NodeType::Window(new_window);
        return;
    }

    let (left_dim, right_dim) = match &orientation {
        Orientation::Horizontal => tile_horizontal(&root.dim),
        Orientation::Vertical => tile_vertical(&root.dim),
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
        }
        NodeType::Separator(Orientation::Vertical, left_child, right_child) => {
            let (left, right) = tile_vertical(&root.dim);
            left_child.dim = left;
            right_child.dim = right;
            resize_children::<T>(left_child);
            resize_children::<T>(right_child);
        }
        _ => return,
    }
}

pub fn find_node<T: Clone + PartialEq>(root: &mut Node<T>, window: T) -> Option<&mut Node<T>> {
    if let NodeType::Window(win) = &root.node_type {
        if win == &window {
            return Some(root);
        } else {
            return None;
        }
    }

    if let NodeType::Separator(_, left_win, right_win) = &mut root.node_type {
        let mut node = find_node::<T>(left_win, window.clone());
        if let None = &mut node {
            node = find_node::<T>(right_win, window.clone());
        }
        return node;
    }

    return None;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tile_vertical_splits_dimensions_in_half() {
        let base_dim = Dimensions {
            x: (0, 1920),
            y: (0, 1080)
        };

        let (left_dim, right_dim) = tile_vertical(&base_dim);

        assert_eq!((0, 960), left_dim.x);
        assert_eq!((0, 1080), left_dim.y);
        assert_eq!((961, 960), right_dim.x);
        assert_eq!((0, 1080), right_dim.y);
    }

    #[test]
    fn tile_horizontal_splits_dimensions_in_half() {
        let base_dim = Dimensions {
            x: (0, 1920),
            y: (0, 1080)
        };

        let (top_dim, bot_dim) = tile_horizontal(&base_dim);

        assert_eq!((0, 1920), top_dim.x);
        assert_eq!((0, 540), top_dim.y);
        assert_eq!((0, 1920), bot_dim.x);
        assert_eq!((541, 540), bot_dim.y);
    }

    #[test]
    fn untile_right_leaf_should_make_left_leaf_root() {
        let left_leaf: Node<i32> = Node {
            node_type: NodeType::Window(1),
            dim: Dimensions {
                x: (0, 960),
                y: (0, 1080)
            }
        };
        let right_leaf: Node<i32> = Node {
            node_type: NodeType::Window(2),
            dim: Dimensions {
                x: (961, 960),
                y: (0, 1080)
            }
        };

        let mut root: Node<i32> = Node {
            node_type: NodeType::Separator(Orientation::Vertical, Box::new(left_leaf), Box::new(right_leaf)),
            dim: Dimensions {
                x: (0, 1920),
                y: (0, 1080)
            }
        };

        untile(&mut root, &2);
        if let NodeType::Window(val) = root.node_type {
            assert_eq!(val, 1);
        } else {
            panic!("Root is not a Window type");
        }
    }

    #[test]
    fn untile_left_leaf_should_make_right_leaf_root() {
        let left_leaf: Node<i32> = Node {
            node_type: NodeType::Window(1),
            dim: Dimensions {
                x: (0, 960),
                y: (0, 1080)
            }
        };
        let right_leaf: Node<i32> = Node {
            node_type: NodeType::Window(2),
            dim: Dimensions {
                x: (961, 960),
                y: (0, 1080)
            }
        };

        let mut root: Node<i32> = Node {
            node_type: NodeType::Separator(Orientation::Vertical, Box::new(left_leaf), Box::new(right_leaf)),
            dim: Dimensions {
                x: (0, 1920),
                y: (0, 1080)
            }
        };

        untile(&mut root, &1);
        if let NodeType::Window(val) = root.node_type {
            assert_eq!(val, 2);
        } else {
            panic!("Root is not a Window type");
        }
    }

    #[test]
    fn tile_should_create_vertical_root_with_children() {
        let mut root: Node<i32> = Node {
            node_type: NodeType::Window(1),
            dim: Dimensions {
                x: (0, 960),
                y: (0, 1080)
            }
        };

        tile(&mut root, Orientation::Vertical, 2);

        if let NodeType::Separator(Orientation::Vertical, left, right) = root.node_type {
            if let NodeType::Window(val) = left.node_type {
                assert_eq!(val, 1);
            } else {
                panic!("left child is not type window");
            }

            if let NodeType::Window(val) = right.node_type {
                assert_eq!(val, 2);
            } else {
                panic!("left child is not type window");
            }
        } else {
            panic!("Vertical separator is not the new root of the subtree.");
        }
    }

    #[test]
    fn tile_should_create_vertical_root_with_dimensions_totaling_root() {
        let mut root: Node<i32> = Node {
            node_type: NodeType::Window(1),
            dim: Dimensions {
                x: (0, 1920),
                y: (0, 1080)
            }
        };

        tile(&mut root, Orientation::Vertical, 2);

        if let NodeType::Separator(Orientation::Vertical, left, right) = root.node_type {
            let left_dim = left.dim;
            assert_eq!(left_dim, Dimensions {
                x: (0, 960),
                y: (0, 1080)
            });

            let right_dim = right.dim;
            assert_eq!(right_dim, Dimensions {
                x: (961, 960),
                y: (0, 1080)
            });
        } else {
            panic!("Vertical separator is not the new root of the subtree.");
        }
    }

    #[test]
    fn tile_should_create_horizontal_root_with_children() {
        let mut root: Node<i32> = Node {
            node_type: NodeType::Window(1),
            dim: Dimensions {
                x: (0, 960),
                y: (0, 1080)
            }
        };

        tile(&mut root, Orientation::Horizontal, 2);

        if let NodeType::Separator(Orientation::Horizontal, left, right) = root.node_type {
            if let NodeType::Window(val) = left.node_type {
                assert_eq!(val, 1);
            } else {
                panic!("left child is not type window");
            }

            if let NodeType::Window(val) = right.node_type {
                assert_eq!(val, 2);
            } else {
                panic!("left child is not type window");
            }
        } else {
            panic!("Horizontal separator is not the new root of the subtree.");
        }
    }

    #[test]
    fn tile_should_create_horizontal_root_with_dimensions_totaling_root() {
        let mut root: Node<i32> = Node {
            node_type: NodeType::Window(1),
            dim: Dimensions {
                x: (0, 1920),
                y: (0, 1080)
            }
        };

        tile(&mut root, Orientation::Horizontal, 2);

        if let NodeType::Separator(Orientation::Horizontal, left, right) = root.node_type {
            let left_dim = left.dim;
            assert_eq!(left_dim, Dimensions {
                x: (0, 1920),
                y: (0, 540)
            });

            let right_dim = right.dim;
            assert_eq!(right_dim, Dimensions {
                x: (0, 1920),
                y: (541, 540)
            });
        } else {
            panic!("Horizontal separator is not the new root of the subtree.");
        }
    }
}
