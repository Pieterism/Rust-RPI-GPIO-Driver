
//check on condition of something
pub fn run() {
    let age: u8 = 18;
    let check_id: bool = true;
    let knows_person_of_age = true;

    // if else 
    if (age >= 21 && check_id) || knows_person_of_age {
        println!("Barteder: What would you like ot drink?");
    } else if age < 21 && check_id {
        println!("Barteder: Sorry you have to leave");
    } else {
        println!("Bartender: I'll need to see your id");
    }

    //shorthand if
    let is_of_age = if age >= 21 {true} else {false};
    println!("Is of age {}", is_of_age );
}