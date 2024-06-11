use bevy_app::{App, First, FixedPreUpdate};
use bevy_ecs::{
    component::Component,
    system::{Query, ReadOnlySystemParam, StaticSystemParam, SystemParam},
};

/// Defines a component that can be registered to automatically track time.
/// 
/// If this type is registered in an app using the [register_time_tracker](TimeTrackingAppExtension::register_time_tracker) it will automatically get update 
pub trait TimeTracking {
    /// If set to `true` this component will be registered to update after [`Time<Real>`](crate::real::Real) and [`Time<Real>`](crate::real::Real) in the [First] schedule.
    const DOES_UPDATE: bool;
    /// If set to `true` this component will be registered to update after [`Time<Fixed>`](crate::fixed::Fixed) in the [FixedPreUpdate] schedule.
    const DOES_FIXED_UPDATE: bool;

    /// The time source this time tracker tracker tracks in [`update`](TimeTracking::update).
    type UpdateSource<'w>: ReadOnlySystemParam;

    /// The time source this time tracker tracker tracks in [`fixed_update`](TimeTracking::fixed_update).
    type FixedUpdateSource<'w>: ReadOnlySystemParam;

    /// Updates this time tracker with the data from the [UpdateSource](TimeTracking::UpdateSource).
    /// 
    /// If this time tracker was registered with [register_time_tracker](TimeTrackingAppExtension::register_time_tracker) this will be called in the [`First`] schedule.
    fn update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::UpdateSource<'a> as SystemParam>::Item<'_, '_>,
    );

    /// Updates this time tracker with the data from the [FixedUpdateSource](TimeTracking::FixedUpdateSource).
    /// 
    /// This function is intended for trackers which will be read in [FixedUpdate](bevy_ecs::FixedUpdate).
    /// If this time tracker was registered with [register_time_tracker](TimeTrackingAppExtension::register_time_tracker) this will be called in the [`FixedPreUpdate`] schedule.
    fn fixed_update<'a: 'b, 'b>(
        &mut self,
        time: &'b <Self::FixedUpdateSource<'a> as SystemParam>::Item<'_, '_>,
    );
}

pub trait TimeTrackingAppExtension: sealed::Seal {
    fn register_time_tracker<T>(&mut self)
    where
        T: TimeTracking + Component;
}

mod sealed {
    /// No you don't
    pub trait Seal {}
    impl Seal for bevy_app::App {}
}

impl TimeTrackingAppExtension for App {
    fn register_time_tracker<T: TimeTracking + Component>(&mut self) {
        bevy_ecs::system::assert_is_system(update_time_tracker::<T>);
        if T::DOES_UPDATE {
            // self.add_systems(First, update_time_tracker::<T>.after(time_system).in_set(TimeSystem));
            todo!()
        }

        if T::DOES_FIXED_UPDATE {
            todo!()
        }
    }
}

fn update_time_tracker<'a, T>(time: StaticSystemParam<T::UpdateSource<'a>>, mut us: Query<&mut T>)
where
    T: TimeTracking + Component,
    T::UpdateSource<'a>: ReadOnlySystemParam,
{
    let inner = time.into_inner();
    for mut us in &mut us {
        us.update(&inner)
    }
}

fn update_time_tracker_fixed_update<'a, T>(
    time: StaticSystemParam<T::FixedUpdateSource<'a>>,
    mut us: Query<&mut T>,
) where
    T: TimeTracking + Component,
{
    let inner = time.into_inner();
    for mut us in &mut us {
        us.fixed_update(&inner)
    }
}

// use crate::Real;
// use bevy_time_derive::TimeTracking;

// use crate as bevy_time;
// use crate::{UpdatingStopWatch, UpdatingTimer, Virtual};
// #[derive(TimeTracking)]
// pub enum TestEnum {
//     Hello,
//     Variant {
//         a: u32,
//         #[time_tracking(ignore)]
//         b: f64,
//     },
//     Tuple(bool, #[time_tracking(ignore)] bool),
// }

// #[derive(TimeTracking)]
// pub struct Test {
//     a: UpdatingTimer<Virtual>,
//     b: UpdatingTimer<Real>,
//     c: UpdatingTimer<Virtual>,
//     #[time_tracking(ignore)]
//     have_fun: bool,
// }

// #[derive(TimeTracking)]
// pub struct Instance;

// #[derive(TimeTracking)]
// pub struct TestTuple(u64, u32);

// #[derive(TimeTracking)]
// pub struct TestLarge(
//     [u8; 1],
//     [u8; 2],
//     [u8; 3],
//     [u8; 4],
//     [u8; 5],
//     [u8; 6],
//     [u8; 7],
//     [u8; 8],
//     [u8; 9],
//     [u8; 10],
//     [u8; 11],
//     [u8; 12],
//     [u8; 13],
//     [u8; 14],
//     [u8; 15],
//     [u8; 16],
//     [u8; 17],
//     [u8; 18],
//     [u8; 19],
//     [u8; 20],
// );
