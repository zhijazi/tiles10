use crate::core::Dimensions;

enum NodeType {
    Horizontal,
    Vertical,
    Window(Node)
}

struct Node {
    node_type: Box<NodeType>,
    left: Box<NodeType>,
    right: Box<NodeType>
}


pub fn tile_vertical(dim: core::Dimensions) -> (core::Dimensions, core::Dimensions) {
    let left_dim = core::Dimensions {
        x: (dim.x.0, dim.x.1 / 2),
        y: (dim.y.0, dim.y.1)
    };
    let right_dim = core::Dimensions {
        x: (dim.x.0 + dim.x.1 / 2, dim.x.1 / 2),
        y: (dim.y0, dim.y.1)
    }
    (left_dim, right_dim)
}

pub fn tile_horizontal(dim: core::Dimensions) -> (core::Dimensions, core::Dimensions) {
    let top_dim = core::Dimensions {
        x: (dim.x.0, dim.x.1),
        y: (dim.y.0, dim.y.1 / 2)
    };
    let bottom_dim = core::Dimensions {
        x: (dim.x.0, dim.x.1),
        y: (dim.y.0 + dim.y.1 / 2, dim.y.1 / 2)
    };
    (top_dim, bottom_dim)
}

#[cfg(test)]
mod tests {
    #[test]
    fn tile_vertical_success_split_x_even() {
        let dim = core::Dimensions {
            x: (0, 1920),
            y: (0, 1080)
        };

        let (left, right) = tile_vertical(dim);

        // TODO impl Eq on Dimension to make comparisons easier
        assert_eq!(left.x.0, 0);
        assert_eq!(left.x.1, 960);
        assert_eq!(right.x.0, 960);
        assert_eq!(right.x.1, 960);
    }

}
