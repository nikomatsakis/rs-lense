/// DSL for defining lense types
#[macro_export]
macro_rules! lense_dsl {
    ($modes:tt) => {};

    (@DSL $($tt:tt)*) => {
        lense_dsl!((private [] -) $($tt)*);
    };

    (@as_item $item:item) => { $item };
    (@as_expr $expr:expr) => { $expr };

    (@construct (public [$($attr:meta)*] $ext:tt) $($tt:tt)*) => {
        lense_dsl!(@as_item $(#[$attr])* pub $($tt)*);
    };

    (@construct (private [$($attr:meta)*] $ext:tt) $($tt:tt)*) => {
        lense_dsl!(@as_item $(#[$attr])* $($tt)*);
    };


    //////////////////
    // Sub-munchers //
    //////////////////

    /////////
    // type

    (@type $modes:tt $ident:ident<S> = $ty:ty) => {
        lense_dsl!(@construct $modes type $ident<S> = $ty;);
    };

    ///////////
    // struct

    (@struct $modes:tt $ident:ident ($($tt:tt)*) $impls:tt $types:tt) => {
        lense_dsl!(@construct $modes struct $ident<S> where S: $crate::Ref { $($tt)* });
        lense_dsl!(@impl struct $ident $impls);
        lense_dsl!(@impl sized $modes struct $ident $types);
    };

    // #[meta]

    (@struct $modes:tt $ident:ident ($($fields:tt)*) $impls:tt $types:tt
        #[$attr:meta] $($tt:tt)*
    ) => {
        lense_dsl!(@struct $modes $ident
                ($($fields)* #[$attr])
                $impls $types
                $($tt)*
            );
    };

    // pub

    (@struct $modes:tt $ident:ident ($($fields:tt)*) $impls:tt $types:tt
        pub $($tt:tt)*
    ) => {
        lense_dsl!(@struct $modes $ident
                ($($fields)* pub)
                $impls $types
                $($tt)*
            );
    };

    // field: @lense

    (@struct $modes:tt $ident:ident ($($fields:tt)*) ($cursor:ident $($impls:tt)*) ($($types:tt)*)
        $field:ident: @$ty:ident , $($tt:tt)*
    ) => {
        lense_dsl!(@struct $modes $ident
                ($($fields)* $field: $ty<S>,)
                ($cursor $($impls)* $field: try!($ty::lense($cursor)),)
                ($($types)* + <$ty<S>>::size())
                $($tt)*
            );
    };

    // field: Vec<@lense>

    (@struct $modes:tt $ident:ident ($($fields:tt)*) ($cursor:ident $($impls:tt)*) ($($types:tt)*)
        $field:ident: Vec<@$ty:ident> , $($tt:tt)*
    ) => {
        lense_dsl!(@struct $modes $ident
                ($($fields)* $field: <Vec<$ty<S>> as Lense<S>>::Ret,)
                ($cursor $($impls)* $field: try!(<Vec<$ty<S>>>::lense($cursor)),)
                ($($types)* + <Vec<$ty<S>> as SizedLense>::size()) // Intentional error
                $($tt)*
            );
    };

    // field: ty

    (@struct $modes:tt $ident:ident ($($fields:tt)*) ($cursor:ident $($impls:tt)*) ($($types:tt)*)
        $field:ident: $ty:ty , $($tt:tt)*
    ) => {
        lense_dsl!(@struct $modes $ident
                ($($fields)* $field: <$ty as Lense<S>>::Ret,)
                ($cursor $($impls)* $field: try!(<$ty>::lense($cursor)),)
                ($($types)* + <$ty>::size())
                $($tt)*
            );
    };

    /////////
    // Enum

    (@enum $modes:tt $ident:ident ($($tt:tt)*) $impls:tt $types:tt) => {
        lense_dsl!(@construct $modes enum $ident<S> where S: $crate::Ref { $($tt)* });
        lense_dsl!(@impl enum $modes $ident $impls);
        lense_dsl!(@impl sized $modes enum $ident $types);
    };

    // #[meta]

    (@enum $modes:tt $ident:ident ($($variants:tt)*) $impls:tt $types:tt
        #[$attr:meta] $($tt:tt)*
    ) => {
        lense_dsl!(@enum $modes $ident
                ($($variants)* #[$attr])
                $impls $types
                $($tt)*
            );
    };

    // Variant(@Ty, ..)

    // Not yet supported
    // [mixing]   V(@ty, ty)
    // [record]   V { #[attrs] field: ty, #[attrs] field: @ty }
    //
    (@enum $modes:tt $ident:ident ($($variants:tt)*) ($cursor:ident $($impls:tt)*) $types:tt
        $variant:ident , $($tt:tt)*
    ) => {
        lense_dsl!(@enum $modes $ident
                ($($variants)* $variant,)
                ($cursor $($impls)* $ident::$variant,)
                $types
                $($tt)*
            );
    };

    (@enum $modes:tt $ident:ident ($($variants:tt)*) ($cursor:ident $($impls:tt)*) ($($types:tt)*)
        $variant:ident($(@$ty:ident),*) , $($tt:tt)*
    ) => {
        lense_dsl!(@enum $modes $ident
                ($($variants)* $variant($($ty<S>),*),)
                ($cursor $($impls)* $ident::$variant($(try!($ty::lense($cursor))),*),)
                ($($types)* (0usize $(+ $ty::size())*),)
                $($tt)*
            );
    };

    (@enum $modes:tt $ident:ident ($($variants:tt)*) ($cursor:ident $($impls:tt)*) ($($types:tt)*)
        $variant:ident($($ty:ty),*) , $($tt:tt)*
    ) => {
        lense_dsl!(@enum $modes $ident
                ($($variants)* $variant($(<$ty as Lense<S>>::Ret),*),)
                ($cursor $($impls)* $ident::$variant($(try!(<$ty>::lense($cursor))),*),)
                ($($types)* (0usize $(+ <$ty>::size())*),)
                $($tt)*
            );
    };


    ////////////////////////////////
    // Implement respective types //
    ////////////////////////////////

    ///////////
    // struct

    (@impl struct $ident:ident ($cursor:ident $($tt:tt)*)) => {
        unsafe impl<S> $crate::Lense<S> for $ident<S> where S: $crate::Ref {
            type Ret = Self;

            fn lense($cursor: &mut $crate::Cursor<S>) -> $crate::Result<Self::Ret> {
                Ok(lense_dsl!(@as_expr $ident { $($tt)* }))
            }
        }
    };

    //////////
    // union

    // Need to know how many bits to fetch for the tag
    (@impl enum ($vis:tt $meta:tt -) $ident:ident ($cursor:ident $($tt:tt)*)) => {
//      unsafe impl<S> $crate::Lense<S> for $ident<S> where S: $crate::Ref {
//          type Ret = Self;

//          fn lense($cursor: &mut $crate::Cursor<S>) -> $crate::Result<Self::Ret> {
//              unimplemented!()
//          }
//      }
    };

    /////////
    // enum

    // Need to know how many bits to fetch for the tag
    (@impl enum ($vis:tt $meta:tt +) $ident:ident ($cursor:ident $($tt:tt)*)) => {
//      unsafe impl<S> $crate::Lense<S> for $ident<S> where S: $crate::Ref {
//          type Ret = Self;

//          fn lense($cursor: &mut $crate::Cursor<S>) -> $crate::Result<Self::Ret> {
//              unimplemented!()
//          }
//      }
    };

    ///////////////
    // SizedLense

    (@impl sized($vis:tt $meta:tt -) $($void:tt)*) => { };

    (@impl sized $modes:tt enum $ident:ident ($($tt:tt)*)) => {
        unsafe impl<S> $crate::SizedLense for $ident<S>
            where S: $crate::Ref
        {
            fn size() -> usize {
                lense_dsl!(@as_expr *[$($tt)*].iter().max().unwrap())
            }
        }
    };

    (@impl sized($vis:tt $meta:tt $test:ident) struct $ident:ident ($($tt:tt)*)) => {
        lense_dsl!(@sized_test($test) $ident);
        unsafe impl<S> $crate::SizedLense for $ident<S>
            where S: $crate::Ref
        {
            fn size() -> usize {
                lense_dsl!(@as_expr $($tt)*)
            }
        }
    };

    (@sized_test($test:ident) $ident:ident) => {
        #[test]
        fn $test() {
            use $crate::{Aligned, Cursor, Lense, SizedLense, Tag};

            let v = Aligned::new(<$ident<&[u8]>>::size() * 3 + 8);
            let ref mut c = Cursor::new(&*v);

            // Define tag for sized enums
            Tag::<u64>::lense(c).unwrap();

            for _ in 0..3 {
                $ident::lense(c).unwrap();
            }

            assert_eq!(c.waste(), 0);
            assert_eq!(c.remaining(), 0);
        }
    };


    ////////////////////////////////////
    // Configure modes for @construct //
    ////////////////////////////////////

    ((private $meta:tt $ext:tt) pub $($tail:tt)*) => {
        lense_dsl!((public $meta $ext) $($tail)*);
    };
    (($vis:tt $meta:tt -) #[sized] $($tail:tt)*) => {
        lense_dsl!(($vis $meta +) $($tail)*);
    };
    (($vis:tt $meta:tt -) #[sized($test:ident)] $($tail:tt)*) => {
        lense_dsl!(($vis $meta $test) $($tail)*);
    };
    (($vis:tt [$($meta:tt)*] $ext:tt) #[$attr:meta] $($tail:tt)*) => {
        lense_dsl!(($vis [$($meta)* $attr] $ext) $($tail)*);
    };


    ////////////////////////////
    // Boot into sub-munchers //
    ////////////////////////////

    /////////
    // type

    ($modes:tt type $ident:ident = Vec<@$lense:ident>; $($tail:tt)*) => {
        lense_dsl!(@type $modes $ident<S> = <Vec<$lense<S>> as Lense<S>>::Ret);
        lense_dsl!((private [] -) $($tail)*);
    };

    ($modes:tt type $ident:ident = $lense:ty; $($tail:tt)*) => {
        lense_dsl!(@type $modes $ident<S> = <$lense as Lense<S>>::Ret);
        lense_dsl!((private [] -) $($tail)*);
    };

    ///////////
    // struct

    ($modes:tt struct $ident:ident { $($tt:tt)* } $($tail:tt)*) => {
        lense_dsl!(@struct $modes $ident
                (/* struct builder */)
                (_cursor /* impl builder */)
                (0usize /* sized builder */)
                $($tt)*
            );
        lense_dsl!((private [] -) $($tail)*);
    };

    /////////////////
    // enum / union

    ($modes:tt enum $ident:ident { $($tt:tt)* } $($tail:tt)*) => {
        lense_dsl!(@enum $modes $ident
                (/* struct builder */)
                (_cursor /* impl builder */)
                (/* sized builder */)
                $($tt)*
            );
        lense_dsl!((private [] -) $($tail)*);
    };
}
