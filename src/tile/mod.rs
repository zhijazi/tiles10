enum NodeType {
    Horizontal,
    Vertical,
    Window(Node)
}

struct Node {
    type: NodeType,
    left: Box<NodeType>,
    right: Box<NodeType>
}

