use crate::{add, cartesian_to_polar_radians_theta};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn angle_comp_rad(a: f32, b: f32) -> f32 {
    //min(abs(a-b), 360-abs(a-b)
    (a-b).abs().min((2.0*PI)-((a-b).abs()))
}
fn dist(p0: (f32, f32), p1: (f32, f32)) -> f32 {
    ((p0.0-p1.0).powi(2) + (p0.1-p1.1).powi(2)).sqrt()
}
fn angle_comp_rad_from_slope(a: f32, b: f32) -> f32 {
    angle_comp_rad(a.atan(), b.atan())
}
fn angle_comp_deg(a: f32, b: f32) -> f32 {
    //min(abs(a-b), 360-abs(a-b)
    (a-b).abs().min(360.0-(a-b).abs())
}
//velocity is units per second!!!
//all angles and stuff in radians
//slope usually in radians
//x,y is meters i think
pub struct LidarLocalizer {
    pub pos: (f32, f32),
    pub heading: f32,
    pub vel: (f32, f32),
    pub ang_vel: f32,
    pub lines: Vec<Line>,
    pub last_time: Instant
}
type SHIFT = (f32, Box<dyn FnOnce(Vec<InstantLine>, &mut LidarLocalizer) -> ()>);
static TOO_LONG_COUNT: AtomicUsize = AtomicUsize::new(0);
static TOO_FAR_COUNT: AtomicUsize = AtomicUsize::new(0);
static TESTS_COUNT: AtomicUsize = AtomicUsize::new(0);
static SECOND_TESTS_COUNT: AtomicUsize = AtomicUsize::new(0);
static GOOD_TESTS_COUNT: AtomicUsize = AtomicUsize::new(0);
impl LidarLocalizer {
    pub(crate) fn new() -> LidarLocalizer {
        LidarLocalizer {//blank at start
            pos: (0.0, 0.0),
            heading: 0.0,
            vel: (0.0, 0.0),
            ang_vel: 0.0,
            lines: vec![],
            last_time: Instant::now()
        }
    }
    pub fn clone_lines(&self) -> Vec<Line> {
        self.lines.iter().map(|it| it.clone())
            .collect()
    }
    const RADIANS_SLOPE_LIMIT: f32 = 10.0f32.to_radians();
    //it's in m/s, you idiot!
    const MOVEMENT_LIMIT: f32 = 1.0;
    fn try_shift<'a>(&self, by: (f32, f32), lines: &Vec<InstantLine>, movement_limit: f32) -> SHIFT {
        let shift = add(self.pos, by);
        //index is of self.lines, 1 is index of lines, 2 is dist to corresponding self.lines line
        let mut best_detections: Vec<(Option<usize>, f32)> = vec![(None, f32::MAX); self.lines.len()];
        //this is a list of the indexes of the InstantLines we didn't find in the known lines
        let mut unfound: Vec<usize> = (0..lines.len()).collect();
        let mut score = 0.0;

        for (index, detection) in lines.iter().enumerate() {
            let rad_slope = detection.known_avg_slope;//actually radians ig
            let midpoint = add(detection.mid_point(), shift);
            TESTS_COUNT.fetch_add(self.lines.len(), Ordering::SeqCst);
            for (test_index, known_line) in self.lines.iter()
                .filter(|it| angle_comp_rad(it.slope_rad, rad_slope) < Self::RADIANS_SLOPE_LIMIT).enumerate() {
                SECOND_TESTS_COUNT.fetch_add(1, Ordering::SeqCst);
                let too_far_along = dist(midpoint, known_line.mid) > known_line.length * 8.1;
                let dist = distance_to_line(known_line.mid, known_line.slope, midpoint);
                let too_far_away = dist > movement_limit;
                if too_far_along && !too_far_away {
                    TOO_FAR_COUNT.fetch_add(1, Ordering::SeqCst);
                }
                if too_far_away /*&& !too_far_along*/ {
                    TOO_LONG_COUNT.fetch_add(1, Ordering::SeqCst);
                }
                if dist < best_detections[test_index].1 && !too_far_along && !too_far_away {
                    GOOD_TESTS_COUNT.fetch_add(1, Ordering::SeqCst);
                    best_detections[test_index].0 = Some(index);
                    best_detections[test_index].1 = dist;
                }
            }
        }
        let mut lines_found = 0f32;
        for (line, dist) in &best_detections {
            match line {
                None => {}
                Some(index) => {
                    score += dist;
                    lines_found += 1.0;
                    let index = unfound.iter().position(|it| *it == *index);
                    match index {
                        None => {}
                        Some(it) => {unfound.remove(it);}
                    };
                }
            }
        }
        score = score / lines_found.powf(1.2);//favor ones with many detections
        if lines_found < 2.0 {
            //has to be at least two otherwise it will make absolutely no sense
            //maybe increase this later idk?
            score = f32::MAX;
        }

        let exe = Box::new(move |lines: Vec<InstantLine>, it: &mut LidarLocalizer| {
            it.pos = add(it.pos, by);
            for (norm_index, (new_line_index, dist)) in best_detections.into_iter().enumerate() {
                it.lines[norm_index].detection_tries += 1;
                match new_line_index {
                    None => {}
                    Some(id) => {
                        it.lines[norm_index].detection_strength += 1;
                    }
                }
            }

            //type gymnastics to put every ILine whose id is in unfound
            lines.into_iter().enumerate()
                .filter(|(x, y)| unfound.iter().find(|it| **it == *x).is_some())
                .for_each(|(pos, item)| it.lines.push(item.into_line()));
            //println!("{:?}", best_detections);
            //println!("{:?}", unfound);
            let mut i = 0;
            while i < it.lines.len() {
                let line = &it.lines[i];
                if line.detection_tries == 3 && line.detection_strength < 3 {
                    it.lines.remove(i);
                } else {
                    i += 1;
                }
            }
            it.last_time = Instant::now();
        });
        (score, exe)
    }
    fn test_region<'a>(&mut self, center: (f32, f32), dist: f32, steps: isize, lines: &Vec<InstantLine>, movement_limit: f32) -> Option<((f32, f32), SHIFT)> {
        let mut pos = (0.0, 0.0);
        let mut lowest: SHIFT = (f32::MAX, Box::new(move |_: Vec<InstantLine>, _: &mut LidarLocalizer| {}));
        let step = dist / (steps as f32);
        for x in -steps..=steps {
            let x = x as f32 * step;
            for y in -steps..=steps {
                let y = y as f32 * step;
                let test = self.try_shift(add((x, y), center), lines, movement_limit);
                if test.0 < lowest.0 {
                    lowest = test;
                    pos = (x, y);
                }
            }
        }
        if lowest.0 == f32::MAX {
            None
        } else {
            Some((pos, lowest))
        }
    }
    ///returns the old data, updates everything internally
    pub fn process(&mut self, instant: InstantLidarLocalizer) -> Vec<Line> {
        let seconds = (self.last_time.elapsed().as_millis() as f32) / 1000f32;
        self.pos.0 += self.vel.0 * seconds;
        self.pos.1 += self.vel.1 * seconds;
        let movement_limit = Self::MOVEMENT_LIMIT * seconds;
        let mut last_center: Option<((f32, f32), SHIFT)> = Some(((0.0, 0.0), (f32::MAX, Box::new(move |_: Vec<InstantLine>, _: &mut LidarLocalizer| {}))));
        let lines = &instant.lines;
        let mut first = 6;
        for i in 0..5 {
            match &last_center {
                None => {first = i.min(first);}
                Some((center, shift)) => {
                     last_center = self.test_region(*center, 4.0 / (i as f32).powi(2), 4, lines, movement_limit);
                }
            }
        }
        let fnc = match last_center {
            None => {
                let too_long = TOO_LONG_COUNT.load(Ordering::SeqCst);
                let too_far =  TOO_FAR_COUNT.load(Ordering::SeqCst);
                let tests = TESTS_COUNT.load(Ordering::SeqCst);
                let second_tests = SECOND_TESTS_COUNT.load(Ordering::SeqCst);
                let good_tests = GOOD_TESTS_COUNT.load(Ordering::SeqCst);
                if self.lines.len() > 0 && instant.lines.len() > 0 {
                    println!("fp: {:?}", self.lines[0]);
                    println!("fp2: {:?}", instant.lines[0].as_line());
                }
                println!("could not find valid shift... firstfail{}, tl{}, tf{}, tests{}, 2ndtests{}, goods{}, ml{}", first, too_long, too_far, tests, second_tests, good_tests, movement_limit);
                TOO_LONG_COUNT.store(0, Ordering::SeqCst);
                TOO_FAR_COUNT.store(0, Ordering::SeqCst);
                TESTS_COUNT.store(0, Ordering::SeqCst);
                SECOND_TESTS_COUNT.store(0, Ordering::SeqCst);
                GOOD_TESTS_COUNT.store(0, Ordering::SeqCst);
                self.try_shift((0.0, 0.0), lines, movement_limit).1
            }
            Some(it) => {it.1.1}
        };
        let tmp: Vec<Line> = instant.lines.iter().map(|x| x.as_line()).collect();

        fnc(instant.lines, self);

        self.last_time = Instant::now();
        tmp
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[derive(Clone, Copy)]
pub struct Line {
    pub mid: (f32, f32),
    pub slope: f32,
    pub slope_rad: f32,
    pub length: f32,
    pub p0: (f32, f32),//debug values. start and end positions
    pub p1: (f32, f32),
    ///roughly, how many times this line had been detected
    pub detection_strength: usize,
    pub detection_tries: usize
}

impl Line {}

fn slope(p0: (f32, f32), p1: (f32, f32)) -> f32 {
    (p1.1-p0.1)/(p1.0-p0.0)
}
//WARNING: this code generated by chatgpt. if it starts producing insane data don't touch it.
//TODO: replace this function I think it's fucked - Owen while trying to write his paper and explain the piece of garbage you're currently looking at
//TODO: use this ya lazy idiot https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Line_defined_by_two_points
fn distance_to_line(line_point: (f32, f32), slope: f32, new_point: (f32, f32)) -> f32 {
    let (x1, y1) = line_point;  // Point on the line
    let (x0, y0) = new_point;   // Point to find the distance to
    //TODO TODO TODO you idiot you mixed up slope and radians like, a *bunch* so fix that asap
    // Calculate the numerator: |m(x0 - x1) - (y0 - y1)|
    let numerator = (slope * (x0 - x1)) - (y0 - y1);

    // Calculate the denominator: sqrt(m^2 + 1)
    let denominator = (slope.powi(2) + 1.0).sqrt();

    // Return the absolute value of the numerator divided by the denominator
    numerator.abs() / denominator
}
fn average_max_dist_to_line(line_point: (f32, f32), slope: f32, line_points: &[(f32, f32)]) -> (f32, f32) {
    let mut max: f32 = 0.0;
    let x = line_points.iter()
        .map(|it| {
            let d = distance_to_line(line_point, slope, *it);
            max = max.max(d);
            d
        });
    (x.sum::<f32>() / line_points.len() as f32, max)
}
trait Reducible where Self: Sized {
    fn best(a: &Self, b: &Self) -> bool;//true = first best, false = second best
    fn are_equivalent(&self, o: &Self) -> bool;
}
struct InstantLine {
    points: Vec<(f32, f32)>,
    ///in radians
    known_avg_slope: f32
}
impl Reducible for InstantLine {
    fn best(a: &InstantLine, b: &InstantLine) -> bool {
        if a.dist() > b.dist() {
            true
        } else {
            false
        }
    }
    fn are_equivalent(&self, other: &InstantLine) -> bool {
        if angle_comp_rad(self.known_avg_slope, other.known_avg_slope) > Self::EQU_WITHIN_DEGREES.to_radians() {
            return false;
        }
        for x in &self.points {
            for y in &other.points {
                if *x == *y {
                    return true
                }
            }
        }
        return false
        /*let find = self.points.iter().enumerate().find_map(|(i, it)| {
            let x = other.points.iter().enumerate().find(|(_i2, it2)| it.eq(it2));
            match x {
                None => { None }
                Some((i2, it2)) => { Some((i, i2)) }
            }
        });
        match find {
            None => { false }
            Some(_) => { true },
        }*/
    }
}
impl InstantLine {
    fn mid_point(&self) -> (f32, f32) {
        let mut avg_x = 0.0;
        let mut avg_y = 0.0;
        for point in &self.points {
            avg_x += point.0;
            avg_y += point.1;
        }
        avg_x = avg_x / (self.points.len() as f32);
        avg_y = avg_y / (self.points.len() as f32);
        (avg_x, avg_y)
    }
    fn dist(&self) -> f32 {
        dist(self.points[0], self.points[self.points.len()-1])
    }
    fn as_line(&self) -> Line {
        let mid = self.mid_point();
        let slope = slope(self.points[0], *self.points.last().unwrap());
        let length = dist(self.points[0], *self.points.last().unwrap());
        if length == 0.0 {
            println!("length 0! p0: {:?}, last: {:?} --- len:{}", self.points[0], self.points.last().unwrap(), self.points.len());
            println!("points! {:?}", self.points);
            panic!("ending...");
        }
        Line {mid, slope, slope_rad: slope.atan(), length, p0: self.points[0], p1: *self.points.last().unwrap(), detection_strength: 0, detection_tries: 0}
    }
    fn into_line(self) -> Line {
        self.as_line()
    }
    fn avg_point_dist_from_center(&self) -> f32 {
        let mid = self.mid_point();
        let slope = self.self_avg_slope();
        self.points.iter().map(|it| distance_to_line(mid, slope, *it)).sum::<f32>() / self.points.len() as f32
    }
    pub fn self_avg_slope(&self) -> f32 {
        Self::avg_slope(self.points.as_slice())
    }
    //this is in radians because im an idiot
    fn avg_slope(points: &[(f32, f32)]) -> f32 {
        let dx = points.last().unwrap().0 - points[0].0;
        let dy = points.last().unwrap().1 - points[0].1;
        dy.atan2(dx)
    }
    //this returns the avg amount that each pair of points' slope (in radians) is different from the avg slope
    fn compare_avg_slope(points: &[(f32, f32)], avg_slope: f32) -> f32 {
        let sum: f32 = points
            .windows(2)
            .map(|w| {
                let dx = w[1].0 - w[0].0;
                let dy = w[1].1 - w[0].1;
                angle_comp_rad(dy.atan2(dx), avg_slope)
            })
            .sum();
        sum / (points.len()-1) as f32
    }
    const ALLOWED_INIT_AVG_POINT_DISTANCE: f32 = 0.003;//3 cm
    pub const INIT_LINE_POINTS: usize = 7;
    fn is_line(p: [(f32, f32); Self::INIT_LINE_POINTS]) -> Option<InstantLine> {
        //let mut slopes = [0f32; Self::INIT_LINE_POINTS-1];//must be 1 less than p.len()
        let slope = slope(p[0], *p.last().unwrap());
        let avg_deg_rad = Self::avg_slope(&p);
        let avg_deg_rad_off = Self::compare_avg_slope(&p, avg_deg_rad);
        let (avg_dist, max_dist) = average_max_dist_to_line(p[0], slope, &p);
        let yes = avg_deg_rad_off.abs() < (Self::WITHIN_DEGREES / 180.0 * PI) && avg_dist < Self::ALLOWED_INIT_AVG_POINT_DISTANCE;
        if !yes {
            return None
        }
        //TODO: cache current slope, compare it against other points we might add to solve that slope drift problem. also < computation probably
        //TODO: don't let known_avg slope change later. this stops slope drift from happening, and then we can make point adding criteria much looser
        //TODO: this will require more logic around left and right possibly
        Some(InstantLine {points: p.to_vec(), known_avg_slope: avg_deg_rad })
    }
    const WITHIN_DEGREES: f32 = 12.5;
    const EQU_WITHIN_DEGREES: f32 = 10.0;
    const POINT_DISTANCE: f32 = 0.01;//10 cm. dist between the closest point in list and our new point
    const STRAIGHTNESS: f32 = 0.005;//5 cm. this is now far new points can be from the line between the first and last point
    fn near_far_points_dist(points: &[(f32, f32)], new_point: (f32, f32)) -> ((f32, f32), (f32, f32), f32, f32){
        let p0 = *points.first().unwrap();
        let p1 = *points.last().unwrap();
        let d0 = dist(p0, new_point);
        let d1 = dist(p1, new_point);
        if d0 < d1 {
            (p0, p1, d0, d1)
        } else {
            (p1, p0, d1, d0)
        }
    }
    fn should_add(&mut self, p: &(f32, f32), left: bool) -> bool {
        //NOTE: slopes always left to right. Assume points sorted.
        let (near_point, far_point, new_distance, _) = Self::near_far_points_dist(&self.points, *p);
        let new_slope = (if left {slope(*p, far_point)} else {slope(far_point, *p)}).atan();//what slope will be if we accept in radians
        let old_slope = self.known_avg_slope;//"avg slope" of line so far
        let dist_to_line = distance_to_line(near_point, old_slope.tan(), *p);//
        let to_add = /*angle_comp_rad(old_slope, new_slope) < (Self::WITHIN_DEGREES / 180.0 * PI) && */new_distance < Self::POINT_DISTANCE && dist_to_line < Self::STRAIGHTNESS;
        if to_add {
            if left {
                self.points.insert(0, *p);
            } else {
                self.points.push(*p);
            }
            //self.known_avg_slope = self.self_avg_slope();//TODO remove
        }
        to_add
    }
    fn maybe_add(point: (f32, f32), vec: &mut Vec<(f32, f32)>) {
        match vec.last() {
            None => {vec.push(point)}
            Some(it) => { if *it != point {
                vec.push(point);
            }
            }
        }
    }
    fn combine(&mut self, other: InstantLine) {
        let mut new: Vec<(f32, f32)> = Vec::with_capacity(self.points.len() + other.points.len());
        //assume both sorted :sob:
        let mut index0 = 0usize;
        let mut index1 = 0usize;
        while index0 < self.points.len() || index1 < other.points.len() {
            if index0 == self.points.len() {
                Self::maybe_add(other.points[index1], &mut new);
                index1 += 1;
                continue
            }
            if index1 == other.points.len() {
                Self::maybe_add(self.points[index0], &mut new);
                index0 += 1;
                continue
            }
            let p0 = self.points[index0];
            let p1 = other.points[index1];
            let o0 = cartesian_to_polar_radians_theta(p0.0, p0.1);
            let o1 = cartesian_to_polar_radians_theta(p1.0, p1.1);
            if o0 > o1 {
                Self::maybe_add(p1, &mut new);
                index1 += 1
            } else {
                Self::maybe_add(p0, &mut new);
                index0 += 1
            }
        }
        self.points = new;
        self.known_avg_slope = self.self_avg_slope();
    }
}
pub struct InstantLidarLocalizer {
    altered_point_list: Vec<(f32, f32)>,
    pub lines: Vec<InstantLine>
}
impl InstantLidarLocalizer {
    //velocity is x,y (estimate) current speed in units per second. time is millis since scan started
    pub fn new(vel: (f32, f32), time: f32, points: &Vec<(f32, f32)>) -> InstantLidarLocalizer {
        let time0 = Instant::now();
        //println!("running calculations on {} points!", points.len());
        if points.len() < 10 {
            //empty
            return InstantLidarLocalizer { altered_point_list: vec![], lines: vec![] }
        }
        let size = points.len() as f32;
        let time = time / 1000.0; //fraction of a second the scan lasted
        let scan_vel = (vel.0 * time, vel.1 * time); //distance traveled during scan
        let altered_points: Vec<(f32, f32)> = points.iter().enumerate()
            //index -> fraction of list completed
            .map(|(i, it)| (i as f32 / size, it))
            .map(|(i, it)| (it.0 + (scan_vel.0 * i), it.1 + (scan_vel.1 * i)))
            .collect();
        let mut i = 2usize;
        let mut lines = Vec::new();
        //println!("initial proc took {} ms", time0.elapsed().as_millis());
        while i < altered_points.len() - InstantLine::INIT_LINE_POINTS {
            let line = InstantLine::is_line(altered_points[i..i + InstantLine::INIT_LINE_POINTS].try_into().unwrap()); //test if consecutive points are in a line
            match line {
                Some(mut it) => {
                    let s = i;
                    //keep trying to add points until they don't "fit"
                    //i += 5;//because those points are in this line
                    //now we'll look for everything else in the line
                    for x in (i+InstantLine::INIT_LINE_POINTS)..altered_points.len() {
                        it.should_add(&altered_points[x], false);
                    }
                    for x in (0..s).rev() { //check all the others
                        //reversed because that will keep the line in order
                        it.should_add(&altered_points[x], true);
                    }
                    lines.push(it);
                }
                None => { /*ignore this*/ }
            }
            i += 1;
        }
        let count = lines.iter().filter(|x| x.points[0] == x.points[1]).count();
        //println!("lines done after {} ms. zeros: {}", time0.elapsed().as_millis(), count);
        //should sort in ascending order of distances
        //lines.sort_by(|x, y| y.known_avg_slope.total_cmp(&x.known_avg_slope));
        //lines = lines.into_iter().filter(|it| {
        //    it.points.len() > (InstantLine::INIT_LINE_POINTS as f64 * 1.25) as usize
        //}).collect();
        let len = lines.len();
        lines.reduce();
        let count = lines.iter().filter(|x| x.points[0] == x.points[1]).count();
        //println!("reducing on {} lines done after {} ms (to {} lines), zeros: {}", len, time0.elapsed().as_millis(), lines.len(), count);
        InstantLidarLocalizer { altered_point_list: altered_points, lines }
    }
}
trait Reduce<T> where Self: Sized, T: Reducible {
    fn reduce(&mut self);
}
impl Reduce<InstantLine> for Vec<InstantLine> {
    fn reduce(&mut self) {
        let mut len = self.len();
        for x in 0..len {
            if x >= len {//guard
                continue
            }
            let mut y = x + 1;
            while y < len {
                if self[x].are_equivalent(&self[y]) {
                    len -= 1;
                    if !InstantLine::best(&self[x], &self[y]) {
                        self.swap(x, y);
                    }
                    let old = self.remove(y);
                    self[x].combine(old);
                } else {
                    y += 1;
                }
            }
        }
    }
}