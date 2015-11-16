use {DiceRef, DiceMut, Lense, LenseMut, RefMut, Mode, IsRef, IsMut};

macro_rules! mk_lense_ty {
    (@void $void:tt $expr:expr) => { $expr };

    (tuple $($ty:ident)*) => { mk_lense_ty!{ () void $($ty)* } };
    (array $($tt:tt)*) => { mk_lense_ty!{ [] $(($tt))* } };
    (prim $($ty:ty)*) => {$(
        impl<'a> RefMut<'a> for $ty {
            type Ref = &'a $ty;
            type Mut = &'a mut $ty;
        }

        impl<'a> Lense for $ty {
            type Ref = <$ty as RefMut<'a>>::Ref;

            #[inline]
            fn size() -> usize {
                ::std::mem::size_of::<Self>()
            }

            #[inline]
            #[allow(unused_variables)]
            fn lense<Buf: DiceRef>(buf: &mut Buf) -> Self::Ref {
                buf.dice::<Self>()
            }
        }

        impl<'a> LenseMut for $ty {
            type Mut = <$ty as RefMut<'a>>::Mut;

            #[inline]
            #[allow(unused_variables)]
            fn lense_mut<Buf: DiceMut>(buf: &mut Buf) -> Self::Mut {
                buf.dice_mut::<Self>()
            }
        }
    )*};

    (()) => { };
    (() $head:tt $($tail:ident)*) => {
        impl<$($tail: Lense),*> Lense for ($($tail,)*) {
            type Ref = ($(<$tail as Mode<IsRef>>::Return,)*);

            #[inline]
            fn size() -> usize {
                0usize $(+ <$tail>::size())*
            }

            #[inline]
            #[allow(unused_variables)]
            fn lense<Buf: DiceRef>(buf: &mut Buf) -> Self::Ref {
                ($(<$tail>::lense(buf),)*)
            }
        }

        impl<$($tail: LenseMut),*> LenseMut for ($($tail,)*) {
            type Mut = ($(<$tail as Mode<IsMut>>::Return,)*);

            #[inline]
            #[allow(unused_variables)]
            fn lense_mut<Buf: DiceMut>(buf: &mut Buf) -> Self::Mut {
                ($(<$tail>::lense_mut(buf),)*)
            }
        }
        mk_lense_ty!{ () $($tail)* }
    };

    ([]) => { };
    ([] ($n:expr) $(($m:expr))*) => {
        impl<L: Lense> Lense for [L; $n] {
            type Ref = [<L as Mode<IsRef>>::Return; $n];

            #[inline]
            fn size() -> usize {
                $n * L::size()
            }

            #[inline]
            #[allow(unused_variables)]
            fn lense<Buf: DiceRef>(buf: &mut Buf) -> Self::Ref {
                [$(mk_lense_ty!{ @void ($m) L::lense(buf) }),*]
            }
        }

        impl<L: LenseMut> LenseMut for [L; $n] {
            type Mut = [<L as Mode<IsMut>>::Return; $n];

            #[inline]
            #[allow(unused_variables)]
            fn lense_mut<Buf: DiceMut>(buf: &mut Buf) -> Self::Mut {
                [$(mk_lense_ty!{ @void ($m) L::lense_mut(buf) }),*]
            }
        }
        mk_lense_ty!{ [] $(($m))* }
    };
}

mk_lense_ty!{prim
     u8  i8
    u16 i16
    u32 i32 f32
    u64 i64 f64
}

mk_lense_ty!{tuple
    A B C D E F
    G H I J K L
}

mk_lense_ty!{array
    32 31 30 29 28 27 26 25
    24 23 22 21 20 19 18 17
    16 15 14 13 12 11 10  9
     8  7  6  5  4  3  2  1
     0
}


/// Create a lense-safe struct containing lense-safe types (Enums are experimental)
#[macro_export]
macro_rules! mk_lense_struct {
    (@as_item $item:item) => { $item };

    // Type independant item parsing

    ([$($meta:tt)*] #[$attr:meta] $($tt:tt)*) => {
        mk_lense_struct!{ [$($meta)* $attr] $($tt)* }
    };
    ([$($meta:tt)*] pub $ty:tt $ident:ident: $($tt:tt)*) => {
        mk_lense_struct!{ @$ty public ([$($meta)*] $ident) () $($tt)* }
    };
    ([$($meta:tt)*] $ty:tt $ident:ident: $($tt:tt)*) => {
        mk_lense_struct!{ @$ty private ([$($meta)*] $ident) () $($tt)* }
    };

    // Struct parsing

    (@struct public ([$($meta:tt)*] $ident:ident $($builder_struct:tt)*)
                    ($($field:ident: $ty:ty,)*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            $(#[$meta])* pub struct $ident<M> where $($ty: $crate::Mode<M>),* { $($builder_struct)* }
        }
        mk_lense_struct!{ {} $ident $($field: $ty),* }
    };
    (@struct private ([$($meta:tt)*] $ident:ident $($builder_struct:tt)*)
                     ($($field:ident: $ty:ty,)*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            $(#[$meta])* struct $ident<M> where $($ty: $crate::Mode<M>),* { $($builder_struct)* }
        }
        mk_lense_struct!{ {} $ident $($field: $ty),* }
    };

    (@struct $vis:tt ($($builder_struct:tt)*) ($($builder_impl:tt)*)
        #[$attr:meta] $($tt:tt)*
    ) => {
        mk_lense_struct!{ @struct $vis
            ($($builder_struct)* #[$attr])
            ($($builder_impl)*)
            $($tt)*
        }
    };
    (@struct $vis:tt ($($builder_struct:tt)*) ($($builder_impl:tt)*)
        pub $($tt:tt)*
    ) => {
        mk_lense_struct!{ @struct $vis
            ($($builder_struct)* pub)
            ($($builder_impl)*)
            $($tt)*
        }
    };
    (@struct $vis:tt ($($builder_struct:tt)*) ($($builder_impl:tt)*)
        $ident:ident: $ty:ty , $($tt:tt)*
    ) => {
        mk_lense_struct!{ @struct $vis
            ($($builder_struct)* $ident: <$ty as $crate::Mode<M>>::Return,)
            ($($builder_impl)* $ident: $ty,)
            $($tt)*
        }
    };

    // Enum parsing

    (@enum public ([$($meta:tt)*] $ident:ident $($builder_struct:tt)*)
                  ($($field:ident($($ty:ty),*))*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            $(#[$meta])* pub enum $ident<M> where $($($ty: $crate::Mode<M>),*),* {
                InvalidLense,
                $($builder_struct)*
            }
        }
        mk_lense_struct!{ E $ident $($field($($ty),*))* }
    };
    (@enum private ([$($meta:tt)*] $ident:ident $($builder_struct:tt)*)
                   ($($field:ident($($ty:ty),*))*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            enum $ident<M> where $($($ty: $crate::Mode<M>),*),* {
                InvalidLense,
                $($builder_struct)*
            }
        }
        mk_lense_struct!{ E $ident $($field($($ty),*))* }
    };

    (@enum $vis:tt ($($builder_struct:tt)*) ($($builder_impl:tt)*)
        #[$attr:meta] $($tt:tt)*
    ) => {
        mk_lense_struct!{ @enum $vis
            ($($builder_struct)* #[$attr])
            ($($builder_impl)*)
            $($tt)*
        }
    };
    (@enum $vis:tt ($($builder_struct:tt)*) ($($builder_impl:tt)*)
        $ident:ident($ty:ty) , $($tt:tt)*
    ) => {
        mk_lense_struct!{ @enum $vis
            ($($builder_struct)* $ident(<$ty as $crate::Mode<M>>::Return),)
            ($($builder_impl)* $ident($ty))
            $($tt)*
        }
    };

    // Lense struct implementations

    ({} $ident:ident $($field:ident: $ty:ty),* $(,)*) => {
        impl<M> $crate::Lense for $ident<M>
            where $($ty: $crate::Mode<M>),*
        {
            type Ref = $ident<$crate::IsRef>;

            #[inline]
            fn size() -> usize {
                0usize $(+ <$ty>::size())*
            }

            #[inline]
            #[allow(unused_variables)]
            fn lense<Buf: $crate::DiceRef>(buf: &mut Buf) -> Self::Ref {
                $ident::<$crate::IsRef> { $($field: <$ty>::lense(buf)),* }
            }
        }

        impl $crate::LenseMut for $ident<$crate::IsMut> {
            type Mut = $ident<$crate::IsMut>;

            #[inline]
            #[allow(unused_variables)]
            fn lense_mut<Buf: $crate::DiceMut>(buf: &mut Buf) -> Self::Mut {
                $ident { $($field: <$ty>::lense_mut(buf)),* }
            }
        }
    };

    // Enum variant counter

    (@void $void:tt $expr:expr) => { $expr };
    (@count_cont $($elem:tt)*) => { 0u8 $(+ mk_lense_struct!{@void $elem 1u8})* };
    (@count ($($tt:expr),*) $void:tt $($tail:tt)*) => {
        mk_lense_struct!{@count (mk_lense_struct!{@count_cont $($tail)*} $(, $tt)*) $($tail)*}
    };
    (@count $expr:expr) => { $expr };

    // Lense enum implementations

    (E $ident:ident $($variant:ident($($ty:ty),*))*) => {
        impl<M> $crate::Lense for $ident<M>
            where $($($ty: $crate::Mode<M>),*),*
        {
            type Ref = $ident<$crate::IsRef>;

            #[inline]
            fn size() -> usize {
                *[$( <($($ty),*) as $crate::Lense>::size() ),*].iter().max().unwrap()
            }

            #[inline]
            #[allow(non_snake_case)]
            fn lense<Buf: $crate::DiceRef>(buf: &mut Buf) -> Self::Ref {
                let tag = <u8>::lense(buf);
                let ($($variant,)*) = mk_lense_struct!(@count () $( $variant )*);
                match tag {
                    $(x if *x == $variant =>
                        $ident::$variant::<$crate::IsRef>(<($($ty),*)>::lense(buf)), )*
                    _ => $ident::InvalidLense::<$crate::IsRef>,
                }
            }
        }

        impl $crate::LenseMut for $ident<$crate::IsMut> {
            type Mut = $ident<$crate::IsMut>;

            #[inline]
            #[allow(non_snake_case)]
            fn lense_mut<Buf: $crate::DiceMut>(buf: &mut Buf) -> Self::Mut {
                let tag = <u8>::lense(buf);
                let ($($variant,)*) = mk_lense_struct!(@count () $( $variant )*);
                match tag {
                    $(x if *x == $variant =>
                        $ident::$variant(<($($ty),*)>::lense_mut(buf)), )*
                    _ => $ident::InvalidLense,
                }
            }
        }
    };

    // Start parsing

    ($($tt:tt)*) => { mk_lense_struct!{ [] $($tt)* } };
}
