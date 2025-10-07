// Task
// Create a Point type which represents a 2D point (x and y coordinates). This type has to be Copy and Default.
// Create a Polyline type which represents a non-empty set of Points of unknown size. This type has to be Clone and non-Default.

#[derive(Debug, Copy, Clone, Default)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone)]
struct Polyline {
    points: Vec<Point>,
}

impl Polyline {
    fn new(points: Vec<Point>) -> Option<Self> {
        if points.is_empty() {
            None
        } else {
            Some(Polyline { points })
        }
    }

    fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn last(&self) -> Option<&Point> {
        self.points.last()
    }
}

fn main() {
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;
    let p3 = Point::default();

    println!("p1={p1:?}, p2={p2:?}, p3 (default)={p3:?}");
    assert!(Polyline::new(vec![]).is_none());

    let mut line = Polyline::new(vec![p1, Point { x: 3.0, y: 4.0 }]).unwrap();
    println!("Polyline: {line:?}");

    line.add_point(Point { x: 5.0, y: 6.0 });
    println!("Polyline after adding point: {line:?}");
    println!("Last point: {:?}", line.last());

    let clone_line = line.clone();
    println!("Copied line: {clone_line:?}");
}
