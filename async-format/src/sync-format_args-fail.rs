fn main() {
    let mut string = String::new();
    {
        let fmt = format_args!("spawn {}", 4);
        std::fmt::write(&mut string, fmt).unwrap();
    }
    // 猜测 format_args! 在栈上创建了 [ArgumentV1<'a>] 类型的隐藏值
    // 然后返回一个 Arguments<'a> 类型的值，其内部有前一个值的借用
    // format_args! 的结果只能作为参数传递, 保证使用时, 临时变量未被 drop
}
