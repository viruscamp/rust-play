#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(impl_trait_in_bindings)]

use closure::closure;
use std::rc::Rc;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[macro_export]
macro_rules! ensure_type {
    ( $x:expr, $t:ty ) => {
        {
            let x = $x;
            assert_type!(x, $t);
            x
        }
    };
    ( $t:ty, $x:expr ) => {
        ensure_type!($x, $t)
    };
}

#[macro_export]
macro_rules! assert_type {
    ( $x:expr, $t:ty ) => {
        {
            fn _assert_type(x: &$t) {}
            _assert_type(&$x);
        }
    };
    ( $t:ty, $x:expr ) => {
        assert_type!($x, $t)
    };
}

fn main() {
    println!("Hello, world!");
    let a = Rc::new(3);
    let mut b = Rc::new(8);
    let x: impl FnMut() -> i32 + Clone = closure!(move a, move mut b, || {
        *a + *b
    });
    let x3 = x.clone();

    let mut x = ensure_type!(closure!(|| {
        8
    }), impl FnMut() -> i32);
    let x3 = x.clone();

    print_type_of(&x);

    assert_type!(x, impl FnMut() -> i32);

    assert_type!((), ());

    let x2 = ensure_type!(impl FnMut<()> + Sync + Copy + Clone, closure!(|| {
        "dfd"
    }));
    print_type_of(&x2);

    let i1 = 4;
    print_type_of(&i1);
    let z = ensure_type!(i1, u8);

    let mut i3 = Rc::new(8);
    let bi3 = &mut i3;
    assert_type!(bi3, i32);
    //assert_type!(i32, bi3);
    println!("{}", bi3);

    test_closure_type();
}

fn test_closure() {
    let a = Rc::new(1);
    let mut b = Rc::new(2);
    let c = Rc::new(3);
    let mut d = 43.0;

    let mut f = || {
        println!("a={} b={} c={}", a, &mut b, c);
    };

    let mut f3 = closure!(ref a, ref mut d, || {

    });

    /*
    let mut f = {
        #[inline(always)]
        fn _make_closure<'a>(a: &'a i32, b: &'a mut f64, c: Rc<i32>) -> impl FnMut() -> i32 + 'a {
            move || {
                println!("a={} b={} c={}", a, b, c);
                a + (*b as i32) + *c0
            }
        }
        _make_closure(a, &mut b, &c)
    };

    let mut f = {
        #[inline(always)]
        fn _make_closure(a, &mut b, &c) -> impl FnMut() -> i32 + Sync {
            move || {
                println!("a={} b={} c={}", a, &mut b, c);
            }
        }
        _make_closure(a, &mut b, &c)
    };
    */

    //Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_closure_type() {
    let mut f1: impl FnMut() -> i32 + Clone = || { 3 }; // 闭包本身还实现了 Fn Send
    let mut f2 = f1.clone(); // 类型上写了 Clone 才能用
    //std::thread::spawn(move || { f1() }); // 类型上没写 Send 所以不能用

    let mut f3: impl FnMut<()> + Clone = || { 4 }; // 可以不写返回类型, 但也不能自动推导出来
    let f3z = f3(); // 值存在, 类型未知, 不能当 i32 用

    let mut bf3 = Box::new(f3);
    let bf3z = bf3();


    let mut f4: impl FnMut<()> = f1.clone();
    use_box_closure(Box::new(f4));

    let mut f5: impl FnMut<()> = || { 3.0 };
    use_box_closure(Box::new(f5));

    let mut f6: impl FnMut<()> = || { [1.0, 2.0, 3.0, 4.0] };
    use_box_closure(Box::new(f6));
}

fn use_box_closure(mut bf: Box<impl FnMut<()>>) { // 泛型函数
    let rst = bf();
    print_type_of(&rst);

    let i1: i64 = 333;
    println!("{}", i1);
}

/*
fn test_closure_type2() {
    let i = 48;
    let mut f = 4.0;
    let rc = Rc::new(44);
    // 闭包套闭包是不行的，因为 impl 不让出现在闭包返回类型
    let fn1 = (|i, f, rc| -> impl FnMut() -> i32 + Sync {
        move || { i + f.into() + rc }
    })(&i, &mut f, rc);
}
*/

fn test_closure_auto() {
    let a = Rc::new(1);
    let mut b = Rc::new(2);
    let c = Rc::new(3);

    let mut f = || {
        let a = a;
        let b = &mut b;
        let c = &c;
        println!("a={} b={} c={}", a, b, c);
    };

    //Fn::call(&f, ());
    //FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_closure_move_auto() {
    let mut a = Rc::new(1);
    let mut b = Rc::new(2);
    let c = Rc::new(3);

    let mut f = move || {
        println!("a={} b={} c={}", a, b, c);
    };

    Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_closure_move_force() {
    let a = Rc::new(1);
    let mut b = Rc::new(2);
    let c = Rc::new(3);

    let mut f = move || {
        let a = a;
        println!("a={} b={} c={}", a, b, c);
    };

    //Fn::call(&f, ());
    //FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_once_move() {
    let mut a = Box::new(1);

    let mut f = move || {
        println!("a={}", a);
        a
    };
    //*a += 2; // use of moved value: `*a`

    //Fn::call(&f, ());
    //FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_mut_move_change() {
    let mut a = Box::new(1);

    let mut f = move || {
        println!("a={}", a);
        *a += 1;
    };
    //*a + 2; // use of moved value: `a`

    //Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_mut_borrow_change() {
    let mut a = Box::new(1);

    let mut f = || {
        println!("a={}", a);
        *a += 1;
    };
    //*a += 2; // use of borrowed `a`

    //Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_mut_borrow_nochange() {
    let mut a = Box::new(1);

    let mut f = || {
        println!("a={}", a);
        let a = &mut a;
    };
    //*a += 2; // use of borrowed `a`

    //Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_fn_move() {
    let mut a = Box::new(1);

    let mut f = move || {
        println!("a={}", a);
    };
    //*a += 2; // use of moved value: `*a`

    Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

fn test_fn_borrow() {
    let mut a = Box::new(1);

    let mut f = || {
        println!("a={}", a);
    };

    Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

use std::collections::HashMap;
use chrono::Utc;
use chrono::DateTime;

pub struct AlarmInfo {
 start_time: DateTime<Utc>,
 end_time: Option<DateTime<Utc>>,
}

// duration_is_over 函数判断是否超时
fn duration_is_over(start: &DateTime<Utc>, end: &DateTime<Utc>) -> bool {
    false
}

/// 定时检查是否超时 i32 代表报警类型
fn interval_alarm(alarms: &mut HashMap<String, HashMap<i32, AlarmInfo>>) {
    alarms.retain(|k, v| { !v.is_empty() });
    for (key, value) in alarms.iter_mut() {
        value.retain(|k, v| {
            if let Some(end) = v.end_time {
                // 判断时间是否超时
                if duration_is_over(&Utc::now(), &end) {
                    // 形成报警记录,并移除当前的记录
                    return false
                }
            } else {
                // 只有开始,且超时不形成记录 移除
                if duration_is_over(&Utc::now(), &v.start_time) {
                    return false
                }
            }
            return true
        })
    }
}