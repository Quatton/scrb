use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Bundle)]
pub struct LockedAxesBundle {
    locked_axes: LockedAxes,
    fall_prevention: FallPrevention,
    rest: Restitution,
    damping: Damping,
}

#[derive(Component)]
pub struct FallPrevention;

impl Default for LockedAxesBundle {
    fn default() -> Self {
        LockedAxesBundle {
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Z
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Y,
            fall_prevention: FallPrevention,
            rest: Restitution::coefficient(0.0),
            damping: Damping {
                linear_damping: 1.0,
                angular_damping: 1.0,
            },
        }
    }
}

impl LockedAxesBundle {
    pub fn player() -> Self {
        LockedAxesBundle {
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Z
                | LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Y
                | LockedAxes::ROTATION_LOCKED_Z,
            fall_prevention: FallPrevention,
            rest: Restitution::coefficient(0.0),
            damping: Damping {
                linear_damping: 1.0,
                angular_damping: 1.0,
            },
        }
    }
}
