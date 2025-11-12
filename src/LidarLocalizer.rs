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
        self.lines.iter().map(|it| it.clone())
            .map(|mut it| {
                it.length = func(it.length);
                it.mid.0 = func(it.mid.0);
                it.mid.1 = func(it.mid.1);
                it.p0.0 = func(it.p0.0);
                it.p0.1 = func(it.p0.1);
                it.p1.0 = func(it.p1.0);
                it.p1.1 = func(it.p1.1);
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
    pub length: f32,
    pub p0: (f32, f32),//debug values
    pub p1: (f32, f32)
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
        Line {mid, slope, length, p0: self.points[0], p1: *self.points.last().unwrap()}
    }
    const ALLOWED_INIT_AVG_POINT_DISTANCE: f32 = 0.005;//5 cm
    pub const INIT_LINE_POINTS: usize = 5;
    fn is_line(p: [(f32, f32); Self::INIT_LINE_POINTS]) -> Option<InstantLine> {
        let mut slopes = [0f32; Self::INIT_LINE_POINTS-1];//must be 1 less than p.len()
        let mut avg = 0.0;
        for i in 0..slopes.len() {
            slopes[i] = slope(p[i], p[i+1]).atan();//0-2pi
            avg += slopes[i];
        }
        avg = avg / slopes.len() as f32;
        let avg_dist = dist(p[0], *p.last().unwrap()) / p.len() as f32;
        let yes = avg.abs() < (Self::WITHIN_DEGREES / 180.0 * PI) && avg_dist < Self::ALLOWED_INIT_AVG_POINT_DISTANCE;
        if !yes {
            return None
        }
        Some(InstantLine {points: p.to_vec()})
    }
    const WITHIN_DEGREES: f32 = 12.5;
    const POINT_DISTANCE: f32 = 0.1;//100 cm
    fn should_add(&mut self, p: &(f32, f32), left: bool) -> bool {
        //NOTE: slopes always left to right. Assume points sorted.
        let near_point = if left { &self.points[0] } else { self.points.last().unwrap() };//closest point
        let far_point = if !left { &self.points[0] } else { self.points.last().unwrap() };//farthest away point
        let new_slope = (if left {slope(*p, *far_point)} else {slope(*far_point, *p)}).atan();//what old slope will be if we accept in radians
        let old_slope = slope(self.points[0], *self.points.last().unwrap()).atan();//"avg slope" of line so far
        let new_distance = dist(*p, *near_point);//we will test if within 50 cm
        let to_add = (old_slope-new_slope).abs() < (Self::WITHIN_DEGREES / 180.0 * PI) && new_distance < Self::POINT_DISTANCE;
        if to_add {
            if left {
                self.points.insert(0, *p);
            } else {
                self.points.push(*p);
            }
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
        while i < altered_points.len()-5 {
            let line = InstantLine::is_line(altered_points[i..i+InstantLine::INIT_LINE_POINTS].try_into().unwrap());//test if consecutive points are in a line
            match line {
                Some(mut it) => {
                    let s = i;
                    //keep trying to add points until they don't "fit"
                    i += 5;//because those points are in this line
                    while i < altered_points.len() && it.should_add(&altered_points[i], false) {//left false because these are after
                        i += 1;
                    }//ok so now ideally we've found all the ones in the line, those we won't look over again
                    //now we'll look for anything else in the line
                    for x in i..altered_points.len() {
                        it.should_add(&altered_points[x], false);
                    }
                    for x in (0..s).rev() {//check all the others
                        //reversed because that will keep the line in order
                        it.should_add(&altered_points[x], true);
                    }
                    lines.push(it);
                }
                None => {/*ignore this*/}
            }
            i += 1;
        }
        InstantLidarLocalizer { altered_point_list: altered_points, lines }
    }
}