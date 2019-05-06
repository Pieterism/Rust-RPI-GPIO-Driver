pub fn run() {
    //default this would be i32
    let x = 1;

    //default is f64
    let y = 2.5;

    //explicite type
    let z : i64 = 32421423;

    //max values
    println!("{}", std::i32::MAX);
    println!("{}", std::i64::MAX);

    //setting a boolean
    let active : bool=true;
    println!("is this true? {}", active);

    //get boolean from expression
    let is_greater = 10 > 5;
    println!("is this greater? {}", is_greater);

    //single char in unicode
    let a1 = 'a';
    println!("{}", a1);
}