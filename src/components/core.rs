use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Bundle)]
pub struct LockedAxesBundle {
    locked_axes: LockedAxes,
    fall_prevention: FallPrevention,
}

#[derive(Component)]
pub struct FallPrevention;

impl Default for LockedAxesBundle {
    fn default() -> Self {
        LockedAxesBundle {
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Z,
            fall_prevention: FallPrevention,
        }
    }
}

impl LockedAxesBundle {
    pub fn player() -> Self {
        LockedAxesBundle {
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Z
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Z,
            fall_prevention: FallPrevention,
        }
    }
}
