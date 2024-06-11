use crate::{Fixed, TimeTracker, UpdatingStopwatch, UpdatingTimer, Virtual};



enum TrackedTime {
    Virtual,
    Fixed,
}
pub struct MixedTimer {
    fixed: UpdatingTimer<Fixed>,
    virt: UpdatingTimer<Virtual>,
    tracked: TrackedTime,
}

impl TimeTracker for MixedTimer {
    const DOES_UPDATE: bool = true;

    const DOES_FIXED_UPDATE: bool = true;

    type UpdateSource<'w> = <UpdatingTimer<Virtual> as TimeTracker>::UpdateSource<'w>;

    type FixedUpdateSource<'w> = <UpdatingTimer<Fixed> as TimeTracker>::UpdateSource<'w>;

    fn update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::UpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        self.virt.update(time);
        self.tracked = TrackedTime::Virtual;
    }

    fn fixed_update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::FixedUpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        self.fixed.update(time);
        self.tracked = TrackedTime::Fixed;
    }
}

pub struct MixedStopwatch {
    fixed: UpdatingStopwatch<Fixed>,
    virt: UpdatingStopwatch<Virtual>,
    tracked: TrackedTime,
}

impl TimeTracker for MixedStopwatch {
    const DOES_UPDATE: bool = true;

    const DOES_FIXED_UPDATE: bool = true;

    type UpdateSource<'w> = <UpdatingStopwatch<Virtual> as TimeTracker>::UpdateSource<'w>;

    type FixedUpdateSource<'w> = <UpdatingStopwatch<Fixed> as TimeTracker>::FixedUpdateSource<'w>;

    fn update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::UpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        self.virt.update(time);
        self.tracked = TrackedTime::Virtual;
    }

    fn fixed_update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::FixedUpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        self.fixed.fixed_update(time);
        self.tracked = TrackedTime::Fixed;
    }
}
