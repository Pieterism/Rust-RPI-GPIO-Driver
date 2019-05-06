use std::mem;
//fixed list where data has same type

pub fn run() {
    let mut numbers : [i32; 4] = [1,2,3,4];

    println!("{:?}", numbers);

    //get singel value
    println!("Single value: {}",numbers[0]);

    //reassign a value
    numbers[2]= 20;
    println!("{:?}", numbers);

    //get array length
    println!("{}", numbers.len());

    //arrays are stack allocated
    println!("array occupies {} bytes", mem::size_of_val(&numbers));

    // get slice from an array
    let slice :&[i32] = &numbers[1..3];
    println!("Slice {:?}", slice);
}