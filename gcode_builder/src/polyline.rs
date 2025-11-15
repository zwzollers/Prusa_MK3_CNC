use std::f64::INFINITY;
use std::f64::NEG_INFINITY;

use dxf::entities::EntityType;
use dxf::point;
use dxf::{Point};

use crate::shape::DXF;

use super::shape::Shape;

use super::almost_eq;

#[derive(Debug)]
pub struct Polylines {
    pub shapes: Vec<Polyline>,
    pub aabb: (Point, Point)
}

impl Polylines {
    pub fn from_dxf(dxf: &DXF) -> Self{

        let shapes: Vec<Polyline> = dxf.shapes.iter().map(|x| Polyline::from_shape(x)).collect();

        let bounds: Vec<(Point, Point)> = shapes.iter().map(|x| x.aabb()).collect();
    
        let mut aabb = (Point::new(INFINITY, INFINITY, 0.0), Point::new(NEG_INFINITY, NEG_INFINITY, 0.0));

        for bound in bounds {
            if bound.1.x > aabb.1.x {
                aabb.1.x = bound.1.x;
            }
            if bound.0.x < aabb.0.x {
                aabb.0.x = bound.0.x;
            }
            if bound.1.y > aabb.1.y {
                aabb.1.y = bound.1.y;
            }
            if bound.0.y < aabb.0.y {
                aabb.0.y = bound.0.y;
            }
        }

        let mut pls = Polylines { shapes, aabb };

        pls.heirachy();

        pls
    }

    /// returns a list of intersection points between a line and all polylines
    pub fn find_intersections(&self, line: (&Point, &Point)) -> Vec<Point> {
        let mut points = Vec::new();

        for pl in &self.shapes {
            for i in 0..pl.points.len()-1 {
                let p1 = &pl.points[i];
                let p2 = &pl.points[i+1];

                if let Some(point) = line_intersect((p1,p2), line) {
                    points.push(point);
                }
            }
        }
        points
    }

    /// marks the nest level of each polyline in context to every other polyline
    fn heirachy(&mut self) {
        for pl_test in 0..self.shapes.len() {

            // line to test intersections with other shapes
            let test_line = (self.shapes[pl_test].points.first().unwrap(), &point!(self.aabb.1.x, self.shapes[pl_test].points.first().unwrap().y));

            let mut nest_cnt = 0;

            for pl in &self.shapes.iter().enumerate().filter(|&(i, _)| i != pl_test).map(|(_, v)| v).collect() as &Vec<&Polyline> {

                // number of intersections per polyline
                let mut int_cnt = 0;

                for i in 0..pl.points.len()-1 {
                    let p1 = &pl.points[i];
                    let p2 = &pl.points[i+1];

                    if line_intersect((p1,p2), test_line).is_some() {
                        int_cnt += 1;
                    }
                }

                // if the number of intersections is odd the shape is nested 
                if int_cnt % 2 == 1 {
                    nest_cnt += 1;
                }
            }
            self.shapes[pl_test].nest = Some(nest_cnt);
        }
    }

}

#[derive(Debug)]
pub struct Polyline {
    pub points: Vec<Point>,
    nest: Option<usize>,
}

impl Polyline {
    pub fn from_shape(shape: &Shape) -> Self {
        let mut pl = Polyline{points: Vec::new(), nest:None};

        for item in &shape.obj {
            let mut points = match item.dxf_entity {
                EntityType::Circle(ref circle) => arc_to_lines(&circle.center, circle.radius, 0.0, 360.0, 60),
                EntityType::Line(ref line) => vec![line.p1.clone(), line.p2.clone()],
                EntityType::Arc(ref arc) => arc_to_lines(&arc.center, arc.radius, arc.start_angle, arc.end_angle, 60),
                _ => panic!(),
            };

            // println!("{points:?}");
            // println!("obj: {:?}", item.dxf_entity);

            if  pl.points.len() == 0 {
                points.reverse();
                pl.points.append(&mut points);
            }
            else if !almost_eq(pl.points.last().unwrap(), points.first().unwrap(), 0.0000001) {
                points.reverse();
                points.remove(0);
                pl.points.append(&mut points);
            }
            else {     
                points.remove(0);
                pl.points.append(&mut points);
                
            }
        }

        // println!();

        assert!(pl.points.len() > 1);

        pl
    }

    /// gets the axis aligned bounding box for a polyline
    pub fn aabb(&self) -> (Point, Point) {
        let mut min = Point::new(INFINITY, INFINITY, 0.0);
        let mut max = Point::new(NEG_INFINITY, NEG_INFINITY, 0.0);

        for p in &self.points {
            if p.x > max.x {
                max.x = p.x;
            }
            if p.x < min.x {
                min.x = p.x;
            }
            if p.y > max.y {
                max.y = p.y;
            }
            if p.y < min.y {
                min.y = p.y;
            }
        }

        (min, max)
    }

    pub fn length(&self) -> f64 {
        let mut len = 0.0;

        if self.points.len() == 0 {
            return 0.0
        }

        for i in 0..self.points.len()-1 {
            len += ((self.points[i+1].x - self.points[i].x).powi(2) + (self.points[i+1].y - self.points[i].y).powi(2)).sqrt();
        }

        len
    }

    pub fn offset(&self, offset: f64) -> Self{

        fn dot(a: &Point, b: &Point) -> f64{
            a.x*b.x + a.y*b.y
        }

        fn det(a: &Point, b: &Point) -> f64{
            a.x*b.y - a.y*b.x
        }

        let mut points = Vec::new();

        for i in 0..self.points.len()-1 {
            let p0 = &self.points[(i as i32-1).rem_euclid(self.points.len()as i32 -1)as usize];
            let p1 = &self.points[(i as i32+0).rem_euclid(self.points.len()as i32 -1)as usize];
            let p2 = &self.points[(i as i32+1).rem_euclid(self.points.len()as i32 -1)as usize];

            let v1 = point!(p0.x - p1.x, p0.y - p1.y);
            let v2 = point!(p2.x - p1.x, p2.y - p1.y);

            let len1 = (v1.x.powi(2) + v1.y.powi(2)).sqrt();
            let len2 = (v2.x.powi(2) + v2.y.powi(2)).sqrt();

            let a = det(&v1, &v2).atan2(dot(&v1, &v2)).to_degrees().rem_euclid(360.0);

            //println!("p0:{p0:?} p1:{p1:?} p2:{p2:?} v1:{v1:?} v2:{v2:?} len1:{len1} len2:{len2} angle: {a}");

            //convex
            if a >= 180.0 {
                let pa = point!(p1.x + (-offset * v1.x / len1), p1.y + (-offset * v1.y / len1));
                let pb = point!(p2.x + (-offset * v2.x / len2), p2.y + (-offset * v2.y / len2));

                let a_start = pa.y.atan2(pa.x).to_degrees().rem_euclid(360.0);
                let a_end = pb.y.atan2(pb.x).to_degrees().rem_euclid(360.0);

                println!("pa:{pa:?} pb:{pb:?} a_start:{a_start} a_end:{a_end}");

                let mut arc_points = arc_to_lines(&p1, offset.abs(), a_end, a_start, 60);

                arc_points.reverse();
                points.append(&mut arc_points);
            }
            //concave
            else {

            }
        }

        Polyline{points: points, nest:None}
    } 
}

/// splits arcs and circles into several line segments
fn arc_to_lines(center: &Point, radius: f64, start: f64, end:f64, segments: u32) -> Vec<Point> {
    let mut points = Vec::new();

    let end = if end == 0.0 {360.0} else {end};

    let arc_angle = (end - start).abs();

    let arc_percentage = arc_angle / 360.0;

    let num_lines = (arc_percentage * segments as f64).ceil() as usize;

    let angle_inc = arc_angle / num_lines as f64;

    let mut angle = start;

    for _ in 0..=num_lines {
        
        let n_point = Point::new(
            center.x + (angle.to_radians().cos() * radius),
            center.y + (angle.to_radians().sin() * radius),
            0.0,);

        points.push(n_point);

        angle += angle_inc;
    }

    points
}

/// checks if two lines intersect including the endpoints
fn line_intersect(l1: (&Point, &Point), l2: (&Point, &Point)) -> Option<Point> {

    fn cross(a: &Point, b: &Point) -> f64{
        a.x*b.y - a.y*b.x
    }

    let d1 = point!(l1.1.x-l1.0.x, l1.1.y-l1.0.y);
    let d2 = point!(l2.1.x-l2.0.x, l2.1.y-l2.0.y);

    let d1xd2 = cross(&d1, &d2);

    let t1 = cross(&point!(l2.0.x-l1.0.x, l2.0.y-l1.0.y), &d2) / d1xd2;
    let t2 = cross(&point!(l2.0.x-l1.0.x, l2.0.y-l1.0.y), &d1) / d1xd2;

    if d1xd2 != 0.0 && t1 >= 0.0 && t1 <= 1.0 && t2 >= 0.0 && t2 <= 1.0 {
        return Some(point!(l1.0.x + d1.x*t1, l1.0.y + d1.y*t1));
    }

    None

}