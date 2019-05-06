pub fn run() {
    let mut hello = String::from("hello ");

    println!("length: {}", hello.len());

    //push char
    hello.push('W');

    //push string
    hello.push_str("orld!");

    //capacity in bytes 

    println!("capacity {}", hello.capacity());
    
    //checks if empty
    println!("is empty {}", hello.is_empty());

    //contains a string
    println!("contains world? {}", hello.contains("World"));

    //loop through string split on whitespace
    for word in hello.split_whitespace() {
        println!("{}", word);
    }

    //create string with capacity
    let mut s = String::with_capacity(10);
    s.push('a');
    s.push('b');

    //assertion testing 
    assert_eq!(2, s.len());
    assert_eq!(10, s.capacity());


    println!("{}", s);
}