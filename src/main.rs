fn main() {
    println!("Welcome to Neko Manifest Creator!");

    let args: Vec<String> = std::env::args().collect();
    println!("----------------------------");
    println!("Selected loader: {}", args[2]);
    println!("Selected version: {}", args[3]);
    println!("----------------------------");
}
