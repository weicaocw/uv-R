// Step 02：给 R 的版本号建模。
// 一个 R 版本（如 "1.2.3"、"3.4-1"）本质是"一串整数"，段数不固定，
// 所以我们用一个可增长的列表 Vec 来装这些整数。

#[derive(Debug)]
struct Version {
    parts: Vec<u64>,
}

fn main() {
    let a = Version { parts: vec![1, 2, 3] };
    let b = Version { parts: vec![3, 4] };

    println!("a = {:?}", a);
    println!("b = {:?}", b);
}
