use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    MoveForward,
    MoveBackward,
    TurnLeft,
    TurnRight,
    RunForward,
    RunBackward,
    FastLeftTurn,
    FastRightTurn,
    StrafeLeft,
    StrafeRight,
    FastStrafeLeft,
    FastStrafeRight,
}

impl PlayerAction {
    pub fn default_map() -> InputMap<Self> {
        use KeyCode::*;
        use ModifierKey::*;
        use PlayerAction::*;

        InputMap::default()
            .with_multiple([
                (MoveForward, ArrowUp),
                (MoveBackward, ArrowDown),
                (TurnLeft, ArrowLeft),
                (TurnRight, ArrowRight),
            ])
            .with_multiple([
                (RunForward, ButtonlikeChord::modified(Shift, ArrowUp)),
                (RunBackward, ButtonlikeChord::modified(Shift, ArrowDown)),
                (FastLeftTurn, ButtonlikeChord::modified(Shift, ArrowLeft)),
                (FastRightTurn, ButtonlikeChord::modified(Shift, ArrowRight)),
            ])
            .with_one_to_many(
                StrafeLeft,
                [
                    ButtonlikeChord::from_single(Comma),
                    ButtonlikeChord::modified(Alt, ArrowLeft),
                ],
            )
            .with_one_to_many(
                StrafeRight,
                [
                    ButtonlikeChord::from_single(Period),
                    ButtonlikeChord::modified(Alt, ArrowRight),
                ],
            )
            .with_one_to_many(
                FastStrafeLeft,
                [
                    ButtonlikeChord::modified(Shift, Comma),
                    ButtonlikeChord::from_single(ArrowLeft).with_multiple([Alt, Shift]),
                ],
            )
            .with_one_to_many(
                FastStrafeRight,
                [
                    ButtonlikeChord::modified(Shift, Period),
                    ButtonlikeChord::new([Alt, Shift]).with(ArrowRight),
                ],
            )
    }
}
