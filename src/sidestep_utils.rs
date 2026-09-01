use sidestep::Sidestepper;
use crate::Drivetrain::Drivetrain;

struct SidestepUtil {
	ss: Sidestepper
}
impl SidestepUtil {
	pub fn new(dt: Drivetrain) -> Self {
		let ss = Sidestepper::new(dt);

		Self {ss}
	}
}