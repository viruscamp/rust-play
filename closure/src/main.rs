#![feature(fn_traits)]

use std::rc::Rc;

fn main() {
    println!("Hello, world!");
}

fn test_closure() {
    let a = Rc::new(1);
    let mut b = Rc::new(2);
    let c = Rc::new(3);

    let mut f = || {
        println!("a={} b={} c={}", a, &mut b, c);
    };

    //Fn::call(&f, ());
    FnMut::call_mut(&mut f, ());
    FnOnce::call_once(f, ());
}

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