use std::marker::PhantomData;

use bevy_app::First;
use bevy_ecs::{
    component::Component,
    prelude::{Query, Res},
};

use bevy_ecs::schedule::IntoSystemConfigs;

use crate::{
    time_system, time_tracking::TimeTracking, Fixed, Real, Stopwatch, Time, TimeSystem, Timer,
    Virtual,
};
use bevy_app::FixedPreUpdate;

#[derive(Component, Default, Debug)]
pub struct UpdatingTimer<T> {
    timer: Timer,
    tracking: PhantomData<T>,
}

impl TimeTracking for UpdatingTimer<Real> {
    const DOES_UPDATE: bool = false;

    const DOES_FIXED_UPDATE: bool = true;

    type UpdateSource<'a> = Res<'a, Time<Real>>;

    type FixedUpdateSource<'a> = ();

    fn update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::UpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        todo!()
    }

    fn fixed_update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::FixedUpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        todo!()
    }
}

impl TimeTracking for UpdatingTimer<Virtual> {
    const DOES_UPDATE: bool = false;

    const DOES_FIXED_UPDATE: bool = true;

    type UpdateSource<'a> = Res<'a, Time<Real>>;

    type FixedUpdateSource<'a> = ();

    fn update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::UpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        todo!()
    }

    fn fixed_update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::FixedUpdateSource<'a> as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        todo!()
    }
}

#[derive(Component, Default, Debug)]
pub struct UpdatingStopWatch<T> {
    watch: Stopwatch,
    tracking: PhantomData<T>,
}
