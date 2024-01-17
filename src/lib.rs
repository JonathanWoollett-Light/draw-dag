#![feature(let_chains)]
#![feature(iter_intersperse)]

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::iter::repeat;
use std::ops::Bound::{Excluded, Included};

pub trait Graph: Sized {
    fn next(&self) -> &[Self];
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Equal => self.x.cmp(&other.x),
            ord @ (Ordering::Less | Ordering::Greater) => ord,
        }
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn draw_dag<T: Graph + Display + Copy>(node: T, width_spacing: usize) -> String {
    let mut column = 0;

    let mut prev_depth = None;
    let mut coordinates = BTreeMap::<Coordinate, T>::new();
    let mut stack = vec![(0, node)];
    while let Some((depth, next)) = stack.pop() {
        let display = next.to_string();

        // Add width
        let width = display.chars().count();

        if let Some(pd) = prev_depth
            && depth <= pd
        {
            column += width_spacing + width + 1;
        }
        prev_depth = Some(depth);

        // Add coordinates of the node
        coordinates.insert(
            Coordinate {
                x: column,
                y: depth,
            },
            next,
        );

        // Add new nodes to stack.
        stack.extend(next.next().iter().copied().map(|n| (depth + 1, n)));
    }

    let mut output = String::new();
    let mut row = 0;
    let mut column = 0;
    for (Coordinate { x, y }, node) in coordinates.iter() {
        let row_diff = y - row;
        if row_diff > 0 {
            column = 0;

            let mut prev_iter = coordinates
                .range((
                    Included(Coordinate { x: 0, y: *y - 1 }),
                    Excluded(Coordinate { x: 0, y: *y }),
                ))
                .map(|(coord, _)| coord)
                .copied()
                .peekable();
            output.push('\n');
            let mut last = 0;
            while let Some(prev) = prev_iter.next() {
                let start = Coordinate { x: prev.x, y: *y };
                let end = match prev_iter.peek() {
                    Some(Coordinate { x, .. }) => Coordinate { x: *x, y: *y },
                    None => Coordinate { x: 0, y: *y + 1 },
                };

                let mut below_iter = coordinates
                    .range((Included(start), Excluded(end)))
                    .map(|(coord, _)| coord)
                    .copied()
                    .peekable();

                if let Some(first) = below_iter.next() {
                    output.extend(repeat(' ').take(prev.x - last));

                    if let Some(second) = below_iter.peek() {
                        assert!(second.y == first.y);

                        output.push('├');
                        output.extend(repeat('─').take(second.x - first.x - 1));

                        while let Some(first_following) = below_iter.next() {
                            if let Some(second_following) = below_iter.peek() {
                                output.push('┬');
                                output.extend(
                                    repeat('─').take(second_following.x - first_following.x - 1),
                                );
                            } else {
                                output.push('┐');
                                last = first_following.x + 1;
                            }
                        }
                    } else {
                        output.push('│');
                        last = first.x + 1;
                    }
                }
            }
            output.push('\n');
        }
        row = *y;

        let column_diff = x - column;
        output.extend(repeat(' ').take(column_diff));
        column = *x;

        let display = node.to_string();
        column += display.chars().count();
        output.push_str(&display);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{alloc, Layout};
    use std::ptr;
    use std::ptr::NonNull;

    #[derive(Debug, Clone, Copy)]
    struct OuterTestNode(NonNull<InnerTestNode>);
    impl std::fmt::Display for OuterTestNode {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", unsafe { self.0.as_ref() })
        }
    }
    impl Graph for OuterTestNode {
        fn next(&self) -> &[Self] {
            unsafe { self.0.as_ref().next.as_slice() }
        }
    }

    #[derive(Debug)]
    struct InnerTestNode {
        next: Vec<OuterTestNode>,
        value: usize,
    }
    impl std::fmt::Display for InnerTestNode {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    fn allocate(value: usize, next: Vec<OuterTestNode>) -> OuterTestNode {
        unsafe {
            let ptr = alloc(Layout::new::<InnerTestNode>()).cast();
            ptr::write(ptr, InnerTestNode { next, value });
            OuterTestNode(NonNull::new(ptr).unwrap())
        }
    }
    #[test]
    fn first() {
        let a = allocate(2, Vec::new());
        let b = allocate(1, vec![a]);

        let graph = draw_dag(b, 1);
        assert_eq!(
            graph,
            "\
            1\n\
            │\n\
            2\
        "
        );
    }

    #[test]
    fn second() {
        let a = allocate(15, Vec::new());
        let b = allocate(14, Vec::new());
        let c = allocate(13, Vec::new());
        let d = allocate(12, vec![a]);
        let e = allocate(121, vec![c, b]);
        let f = allocate(10, vec![d, e]);

        let graph = draw_dag(f, 1);
        assert_eq!(
            graph,
            "\
            10\n\
            ├───────┐\n\
            121     12\n\
            ├───┐   │\n\
            14  13  15\
        "
        );
    }
    #[test]
    fn three() {
        let a = allocate(1, Vec::new());
        let b = allocate(2, Vec::new());
        let c = allocate(3, Vec::new());
        let d = allocate(4, Vec::new());
        let e = allocate(5, vec![a]);
        let f = allocate(6, Vec::new());
        let g = allocate(7, vec![b]);
        let h = allocate(8, vec![d, c]);
        let i = allocate(9, vec![e]);
        let j = allocate(10, vec![h, g, f]);
        let k = allocate(11, vec![j, i]);
        let l = allocate(12, vec![k]);

        let graph = draw_dag(l, 1);
        assert_eq!(
            graph,
            "\
            12\n\
            │\n\
            11\n\
            ├───┐\n\
            9   10\n\
            │   ├──┬──┐\n\
            5   6  7  8\n\
            │      │  ├──┐\n\
            1      2  3  4\
        "
        );
    }
}
