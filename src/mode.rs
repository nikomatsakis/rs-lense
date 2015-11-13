use {Lense, LenseMut};

/// Handle type level modes
pub trait Mode<M> { type Return; }

/// Mode: Immutable reference
pub enum IsRef {}
/// Mode: Mutable reference
pub enum IsMut {}

// RefMut implements both type conditions
impl<E: Lense>    Mode<IsRef> for E { type Return = E::Ref; }
impl<E: LenseMut> Mode<IsMut> for E { type Return = E::Mut; }
