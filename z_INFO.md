# Archetypal components

Archetypal components are components stored in the archetypal storage. The archetypal storage does not exist. 
Anything stored in it will simply vanish into thin air. Any proof of it's possible existence is stored in an
entities archetype. Sound simple enough, right? No!

All over the ecs codebase we assume we can produce a reference to any given component and along 
with this reference we can also provide references to this components change ticks. Archetypal
components break both of these assumptions. Their storage is imaginary and can't store components
or change ticks. If we only allow ZST archetypal components we can forge[^1] references to them but 
that leaves us without a way to the change tick references.

A simple solution would be to panic whenever we try to access a arcehtypal component. For example a system like
```rust
fn panicking_system(stuff: Query<&mut SomeArchetypalComponent>) {...}
```
could panic either when it's query is first fetched or when it's state is first constructed. This doesn't seem like such a good idea especially when we've already got a better solution.

## `RefrencableComponent` and `ChangeTrackingComponent`

By introducing the `RefrencableComponent` and `ChangeTrackingComponent` we can stop our users from

[^1]: It is safe to convert a `NonNull::<T>::dangling()` to a reference if `T` is a  ZST