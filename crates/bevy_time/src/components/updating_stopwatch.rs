use std::fmt::Debug;
use std::marker::PhantomData;

use bevy_ecs::component::Component;

use crate::Stopwatch;

#[derive(Component)]
pub struct UpdatingStopwatch<T> {
    pub watch: Stopwatch,
    tracking: PhantomData<T>,
}

impl<T> UpdatingStopwatch<T> {
    pub fn new(watch: Stopwatch) -> Self {
        Self {
            watch,
            tracking: PhantomData,
        }
    }
}

impl<T> Clone for UpdatingStopwatch<T> {
    fn clone(&self) -> Self {
        Self {
            watch: self.watch.clone(),
            tracking: self.tracking.clone(),
        }
    }
}

impl<T> Debug for UpdatingStopwatch<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdatingStopWatch")
            .field("watch", &self.watch)
            .field("tracking", &self.tracking)
            .finish()
    }
}

impl<T> Default for UpdatingStopwatch<T> {
    fn default() -> Self {
        Self {
            watch: Default::default(),
            tracking: Default::default(),
        }
    }
}

impl<T> PartialEq for UpdatingStopwatch<T> {
    fn eq(&self, other: &Self) -> bool {
        self.watch == other.watch && self.tracking == other.tracking
    }
}

impl<T> Eq for UpdatingStopwatch<T> {}
