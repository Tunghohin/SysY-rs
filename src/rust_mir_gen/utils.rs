pub fn write(s: &str, indent: usize) {
    print!("{}", " ".repeat(indent * 4));
    print!("{}", s);
}

pub fn writeln(s: &str, indent: usize) {
    print!("{}", " ".repeat(indent * 4));
    println!("{}", s);
}
