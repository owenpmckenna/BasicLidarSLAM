use std::f32::consts::PI;
use serde::{Deserialize, Serialize};

//velocity is units per second!!!
//all angles and stuff in radians
//slope usually in radians
//x,y is meters i think
pub struct LidarLocalizer {
    pub pos: (f32, f32),
    pub heading: f32,
    pub vel: (f32, f32),
    pub ang_vel: f32,
    pub lines: Vec<Line>
}
impl LidarLocalizer {
    pub(crate) fn new() -> LidarLocalizer {
        LidarLocalizer {//blank at start
            pos: (0.0, 0.0),
            heading: 0.0,
            vel: (0.0, 0.0),
            ang_vel: 0.0,
            lines: vec![],
        }
    }
    pub fn clone_lines(&self, func: fn(f32) -> f32) -> Vec<Line> {
        self.lines.iter().map(|it| *it)
            .map(|mut it| {
                it.length = func(it.length);
                it.mid.0 = func(it.mid.0);
                it.mid.1 = func(it.mid.1);
                it
            })
            .collect()
    }
    pub fn process(&mut self, instant: InstantLidarLocalizer) {
        //TODO fully redo this method
        self.lines.clear();
        let mut tmp: Vec<Line> = instant.lines.into_iter().map(|x| x.into_line()).collect();
        self.lines.append(&mut tmp);
    }
}
#[derive(Serialize, Deserialize)]
#[derive(Clone, Copy)]
pub struct Line {
    pub mid: (f32, f32),
    pub slope: f32,
    pub length: f32
}
fn slope(p0: (f32, f32), p1: (f32, f32)) -> f32 {
    (p1.1-p0.1)/(p1.0-p0.0)
}
fn dist(p0: (f32, f32), p1: (f32, f32)) -> f32 {
    ((p0.0-p1.0).powi(2) + (p0.1-p1.1).powi(2)).sqrt()
}
struct InstantLine {
    points: Vec<(f32, f32)>
}
impl InstantLine {
    fn into_line(self) -> Line {
        let mut avg_x = 0.0;
        let mut avg_y = 0.0;
        for point in &self.points {
            avg_x += point.0;
            avg_y += point.1;
        }
        avg_x = avg_x / (self.points.len() as f32);
        avg_y = avg_y / (self.points.len() as f32);
        let mid = (avg_x, avg_y);
        let slope = slope(self.points[0], *self.points.last().unwrap());
        let length = dist(self.points[0], *self.points.last().unwrap());
        Line {mid, slope, length}
    }
    const ALLOWED_INIT_AVG_POINT_DISTANCE: f32 = 0.005;//5 cm
    const INIT_LINE_POINTS: usize = 5;//TODO base it off of this
    fn is_line(p: [(f32, f32); 5]) -> Option<InstantLine> {
        let mut slopes = [0f32; 4];//must be 1 less than p.len()
        let mut avg = 0.0;
        for i in 0..slopes.len() {
            slopes[i] = slope(p[i], p[i+1]).atan();
            avg += slopes[i];
        }
        avg = avg / 4.0;
        let dist = dist(p[0], *p.last().unwrap()) / p.len() as f32;
        let yes = avg.abs() < (Self::WITHIN_DEGREES / 180.0 * PI) && dist < Self::ALLOWED_INIT_AVG_POINT_DISTANCE;
        if !yes {
            return None
        }
        Some(InstantLine {points: p.to_vec()})
    }
    const WITHIN_DEGREES: f32 = 7.5;
    const POINT_DISTANCE: f32 = 0.05;//50 cm
    fn should_add(&mut self, p: &(f32, f32), left: bool) -> bool {
        //NOTE: slopes always left to right. Assume points sorted.
        let near_point = if left { &self.points[0] } else { self.points.last().unwrap() };//closest point
        let far_point = if !left { &self.points[0] } else { self.points.last().unwrap() };//farthest away point
        let new_slope = (if left {slope(*p, *far_point)} else {slope(*far_point, *p)}).atan();//what old slope will be if we accept in radians
        let old_slope = slope(self.points[0], *self.points.last().unwrap()).atan();//"avg slope" of line so far
        let new_distance = dist(*p, *near_point);//we will test if within 50 cm
        let to_add = (old_slope-new_slope).abs() < (Self::WITHIN_DEGREES / 180.0 * PI) && new_distance < Self::POINT_DISTANCE;
        if to_add {
            self.points.push(*p);
        }
        to_add
    }
}
pub struct InstantLidarLocalizer {
    altered_point_list: Vec<(f32, f32)>,
    lines: Vec<InstantLine>
}
impl InstantLidarLocalizer {
    //velocity is x,y (estimate) current speed in units per second. time is millis since scan started
    pub fn new(vel: (f32, f32), time: f32, points: &Vec<(f32, f32)>) -> InstantLidarLocalizer {
        if points.len() < 10 {
            //empty
            return InstantLidarLocalizer { altered_point_list: vec![], lines: vec![] }
        }
        let size = points.len() as f32;
        let time = time / 1000.0;//fraction of a second the scan lasted
        let scan_vel = (vel.0 * time, vel.1 * time);//distance traveled during scan
        let altered_points: Vec<(f32, f32)> = points.iter().enumerate()
            //index -> fraction of list completed
            .map(|(i, it)| (i as f32/size, it))
            .map(|(i,it)| (it.0 + (scan_vel.0 * i), it.1 + (scan_vel.1 * i)))
            .collect();
        let mut i = 2usize;
        let mut lines = Vec::new();
        while i < altered_points.len()-2 {
            let line = InstantLine::is_line(altered_points[i-2..i+3].try_into().unwrap());//test if 5 consecutive points are in a line
            match line {
                Some(mut it) => {
                    //keep trying to add points until they don't "fit"
                    i += 2;//because those points are in this line
                    while i < altered_points.len() && it.should_add(&altered_points[i], false) {
                        i += 1;
                    }
                    i += 1;//to go past the points we were looking at...
                    lines.push(it)
                }
                None => {/*ignore this*/}
            }
            i += 1;
        }
        InstantLidarLocalizer { altered_point_list: altered_points, lines }
    }
}