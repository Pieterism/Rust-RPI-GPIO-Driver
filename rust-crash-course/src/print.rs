
pub fn run() {
    //print to the console
    println!("Hello from the print.rs file");

    //basic formatting
    println!("Printing of a number: {}", 1 );

    //possitional arguments
    println!("{0} likes to code and {0} to {1}", "Brad", "play");

    //named arguments
    println!("{name} likes to code and {activity}", name = "Brad", activity = "play" );

    //placeholder traits
    println!("Binary {:b}, Hex:{:x}", 10, 10);

    //Placeholder for debug trait
    println!("{:?}", (12, true, "Hello"));
}