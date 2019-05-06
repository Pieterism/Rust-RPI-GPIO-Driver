pub fn run() {
    let name = "Brad";
    let mut age = 37;
    println!("My name is: {} and i am {}", name, age);

    age = 38;
    println!("My name is: {} and i am {}", name, age);

    //define a const
    const ID: i32 = 001;
    println!("ID: {}", ID);

    //assign multiple values
    let (my_name, my_age) = ("Brad", 37);
    println!("{} {}", my_name, my_age );
}