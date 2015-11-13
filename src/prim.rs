use {DiceRef, DiceMut, Lense, LenseMut, RefMut, Mode, IsRef, IsMut};

macro_rules! mk_lense_ty {
    (@void $void:tt $expr:expr) => { $expr };

    (tuple $($ty:ident)*) => { mk_lense_ty!{ () void $($ty)* } };
    (array $($tt:tt)*) => { mk_lense_ty!{ [] L $(($tt))* } };
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

    ([] $L:ident) => { };
    ([] $L:ident ($n:expr) $(($m:expr))*) => {
        impl<L: Lense> Lense for [$L; $n] {
            type Ref = [<$L as Mode<IsRef>>::Return; $n];

            #[inline]
            fn size() -> usize {
                $n * L::size()
            }

            #[inline]
            #[allow(unused_variables)]
            fn lense<Buf: DiceRef>(buf: &mut Buf) -> Self::Ref {
                [$(mk_lense_ty!{ @void ($m) $L::lense(buf) }),*]
            }
        }

        impl<L: LenseMut> LenseMut for [$L; $n] {
            type Mut = [<$L as Mode<IsMut>>::Return; $n];

            #[inline]
            #[allow(unused_variables)]
            fn lense_mut<Buf: DiceMut>(buf: &mut Buf) -> Self::Mut {
                [$(mk_lense_ty!{ @void ($m) $L::lense_mut(buf) }),*]
            }
        }
        mk_lense_ty!{ [] $L $(($m))* }
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
    (@void $void:tt $expr:expr) => { $expr };
    (@as_item $item:item) => { $item };

    // User level parsing

    (pub struct $ident:ident: $($tt:tt)*) => {
        mk_lense_struct!{ @struct public ($ident) () $($tt)* , }
    };
    (struct $ident:ident: $($tt:tt)*) => {
        mk_lense_struct!{ @struct private ($ident) () $($tt)* , }
    };
    (pub enum $ident:ident: $($tt:tt)*) => {
        mk_lense_struct!{ @enum public ($ident) () $($tt)* , }
    };
    (enum $ident:ident: $($tt:tt)*) => {
        mk_lense_struct!{ @enum private ($ident) () $($tt)* , }
    };

    // Define struct and implementations

    (@struct public ($ident:ident $($builder_struct:tt)*)
                    ($($field:ident: $ty:ty,)*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            pub struct $ident<M> where $($ty: $crate::Mode<M>),* { $($builder_struct)* }
        }
        mk_lense_struct!{ {} $ident $($field: $ty),* }
    };
    (@struct private ($ident:ident $($builder_struct:tt)*)
                     ($($field:ident: $ty:ty,)*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            struct $ident<M> where $($ty: $crate::Mode<M>),* { $($builder_struct)* }
        }
        mk_lense_struct!{ {} $ident $($field: $ty),* }
    };

    // Struct parsing

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

    // Lense implementations

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


    // EXPERIMENTAL ENUM HANDLING!!


    (@enum public ($ident:ident $($builder_struct:tt)*)
                  ($($field:ident($($ty:ty),*))*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            pub enum $ident<M> where $($($ty: $crate::Mode<M>),*),* { InvalidLense, $($builder_struct)* }
        }
        mk_lense_struct!{ E $ident $($field($($ty),*))* }
    };
    (@enum private ($ident:ident $($builder_struct:tt)*)
                   ($($field:ident($($ty:ty),*))*) $(,)*
    ) => {
        mk_lense_struct!{ @as_item
            enum $ident<M> where $($($ty: $crate::Mode<M>),*),* { InvalidLense, $($builder_struct)* }
        }
        mk_lense_struct!{ E $ident $($field($($ty),*))* }
    };

    // Struct parsing

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

    // Lense implementations
    (@count_cont $($elem:tt)*) => { 0u8 $(+ mk_lense_struct!{@void $elem 1u8})* };
    (@count ($($tt:expr),*) $void:tt $($tail:tt)*) => {
        mk_lense_struct!{@count (mk_lense_struct!{@count_cont $($tail)*} $(, $tt)*) $($tail)*}
    };
    (@count $expr:expr) => { $expr };

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
            #[allow(unused_variables)]
            fn lense<Buf: $crate::DiceRef>(buf: &mut Buf) -> Self::Ref {
                let tag = <u8>::lense(buf);
                let ($($variant,)*) = mk_lense_struct!(@count () $( $variant )*);
                let (ret, offset) = match tag {
                    $(x if *x == $variant =>
                        ($ident::$variant::<$crate::IsRef>(<($($ty),*)>::lense(buf)),
                         <($($ty),*)>::size() - Self::size()),)*
                    _ => ($ident::InvalidLense::<$crate::IsRef>, Self::size()),
                };
                debug_assert!(offset == 0, "Enum padding is not yet supported");
                ret
            }
        }

        impl $crate::LenseMut for $ident<$crate::IsMut> {
            type Mut = $ident<$crate::IsMut>;

            #[inline]
            #[allow(unused_variables)]
            fn lense_mut<Buf: $crate::DiceMut>(buf: &mut Buf) -> Self::Mut {
                let tag = <u8>::lense(buf);
                let ($($variant,)*) = mk_lense_struct!(@count () $( $variant )*);
                let (ret, offset) = match tag {
                    $(x if *x == $variant =>
                        ($ident::$variant(<($($ty),*)>::lense_mut(buf)),
                         <($($ty),*)>::size() - Self::size()),)*
                    _ => ($ident::InvalidLense, Self::size()),
                };
                debug_assert!(offset == 0, "Enum padding is not yet supported");
                ret
            }
        }
    }
}
