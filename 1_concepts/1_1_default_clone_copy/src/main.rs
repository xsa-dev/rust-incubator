// Task
// Create a Point type which represents a 2D point (x and y coordinates). This type has to be Copy and Default.
// Create a Polyline type which represents a non-empty set of Points of unknown size. This type has to be Clone and non-Default.

#[derive(Debug, Copy, Clone, Default, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone)]
struct Polyline {
    points: Vec<Point>,
}

impl Polyline {
    pub fn new(points: Vec<Point>) -> Option<Self> {
        if points.is_empty() {
            None
        } else {
            Some(Polyline { points })
        }
    }

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn last(&self) -> Option<&Point> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_supports_copy_and_default() {
        let origin = Point::default();
        assert_eq!(origin, Point { x: 0.0, y: 0.0 });

        let copy = origin;
        assert_eq!(origin, copy, "Copy should duplicate the coordinates");
    }

    #[test]
    fn polyline_requires_points_and_tracks_length() {
        assert!(Polyline::new(vec![]).is_none());

        let mut polyline = Polyline::new(vec![Point { x: 1.0, y: 1.0 }]).unwrap();
        assert_eq!(polyline.len(), 1);
        assert_eq!(polyline.last(), Some(&Point { x: 1.0, y: 1.0 }));

        polyline.add_point(Point { x: -1.0, y: 2.5 });
        assert_eq!(polyline.len(), 2);
        assert_eq!(polyline.last(), Some(&Point { x: -1.0, y: 2.5 }));
    }

    #[test]
    fn cloning_polyline_copies_points() {
        let mut original = Polyline::new(vec![Point { x: 3.0, y: 4.0 }]).unwrap();
        let cloned = original.clone();

        // Changing the original should not affect the clone because points are copied.
        original.add_point(Point { x: 9.0, y: 9.0 });
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned.last(), Some(&Point { x: 3.0, y: 4.0 }));
    }
}
