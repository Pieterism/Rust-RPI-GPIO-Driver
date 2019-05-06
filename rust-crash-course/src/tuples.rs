
//grouping togheter values of different types
// with a max of 12 values 
pub fn run() {
    let person : (&str, &str, i8) = ("Brad", "Mass", 37);
    println!("{} is from {} and is {} years old", person.0, person.1, person.2);
    
}