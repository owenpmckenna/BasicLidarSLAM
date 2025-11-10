//velocity is units per second!!!
//all angles and stuff in radians
//x,y is meters i think
pub struct LidarLocalizer {
    pos: (f32, f32),
    heading: f32,
    vel: (f32, f32),
    ang_vel: f32,
    lines: Vec<Line>
}
struct Line {
    mid: (f32, f32),
    slope: f32,
    length: f32
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
    fn is_line(p: &[(f32, f32)]) -> Option<InstantLine> {
        None
    }
    fn should_add(&self, p: &(f32, f32), left: bool) -> bool {
        let targetPoint = if left { &self.points[0] } else { self.points.last().unwrap() };
        let new_slope = if left {slope(*p, *targetPoint)} else {slope(*targetPoint, *p)};

        false
    }
}
struct InstantLidarLocalizer {
    altered_point_list: Vec<(f32, f32)>,
    lines: Vec<InstantLine>
}
impl InstantLidarLocalizer {
    //velocity is x,y (estimate) current speed in units per second. time is millis since scan started
    fn new(vel: (f32, f32), time: f32, points: Vec<(f32, f32)>) {
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
            let line = InstantLine::is_line(&altered_points[i-2..i+2]);
            match line {
                Some(it) => {
                    i += 2;//because those points are in this line
                    while i < altered_points.len() && it.should_add(&altered_points[i], false) {
                        i += 1;
                    }
                    i += 1;//to go past
                    lines.push(it)
                }
                None => {/*ignore this*/}
            }
            i += 1;
        }
        &altered_points[0..1];
    }
}