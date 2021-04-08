use std::rc::Rc;

fn force_move<T>(v: T) -> T {
    v
}

fn test_fn<T:Fn>(t: &T) {
}
fn test_fn_mut<T:FnMut>(t: &T) {
}
fn test_fn_once<T:FnOnce>(t: &T) {
}

fn main() {
    let i = 32;
    force_move(&i);
    drop(&i);

    let rc = Rc::new(3);
    force_move(&rc);
    drop(&rc);

    let f1 = || {i + *rc};

    test_fn(f1);
    test_fn_mut(f1);
    test_fn_once(f1);
    
    println!("Hello {}", *rc);
}