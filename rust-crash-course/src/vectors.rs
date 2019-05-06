use std::mem;

pub fn run() {
    let mut numbers : Vec<i32> = vec![1,2,3,4];

    println!("{:?}", numbers);

    //get singel value
    println!("Single value: {}",numbers[0]);

    //add on to vector
    numbers.push(5);
    numbers.push(6);

    //reassign a value
    numbers[2]= 20;
    println!("{:?}", numbers);

    //get array length
    println!("{}", numbers.len());

    //Vectors are stack allocated
    println!("array occupies {} bytes", mem::size_of_val(&numbers));

    // get slice from an array
    let slice :&[i32] = &numbers[1..3];
    println!("Slice {:?}", slice);

    // get last value 
    numbers.pop();
    println!("{:?}",numbers );

    // loop through vector values
    for x in numbers.iter() {
        println!("number : {}", x);
    }

    // loop and mutate values -> similar to map
    for x in numbers.iter_mut() {
        *x *= 2;
    }

    
    println!("NUmbers vec: {:?}", numbers);
}