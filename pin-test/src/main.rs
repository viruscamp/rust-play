#![feature(negative_impls)]

use core::marker::PhantomData;
use std::marker::Unpin;
use std::marker::PhantomPinned;

macro_rules! assert_type {
    ($t: ty, $x: expr) => {
        {
            #[inline(always)]
            fn _assert_type(_: &$t) {}
            _assert_type(&$x);
        }
    };
    ($x: expr, $t: ty) => {
        assert_type!($t, $x)
    };
}

#[macro_export]
macro_rules! ensure_type {
    ($t: ty, $x: expr) => {
        {
            let x = $x;
            assert_type!($t, x);
            x
        }
    };
    ($x: expr, $t: ty) => {
        ensure_type!($t, $x)
    };
}

struct St1Unpin {
 i:i32,
}

struct St2NotUnpin {
 i:i32,
 _x:PhantomPinned,
}

struct St3NotUnpin {
 i:i32,
}
impl !Unpin for St3NotUnpin {}

struct St4NotUnpinForceUnpin {
 i:i32,
 _x:PhantomPinned,
}
impl Unpin for St4NotUnpinForceUnpin {}

struct UnpinContainer<T>(T);
impl<T> Unpin for UnpinContainer<T> {}

fn main() {
  let s1 = St1Unpin{i:1};
  assert_type!(Unpin, s1);
  let s2 = St2NotUnpin{i:1, _x: PhantomPinned};
  //assert_type!(Unpin, s2);
  let s3 = St3NotUnpin{i:1};
  //assert_type!(Unpin, s3);
  let s4 = St4NotUnpinForceUnpin{i:1, _x: PhantomPinned};
  assert_type!(Unpin, s4);

  let af1 = async {};
  //assert_type!(Unpin, af1);
  let af2 = UnpinContainer(af1);
  assert_type!(Unpin, af2);
}