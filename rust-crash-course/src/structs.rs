

//traditional struct
//struct Color {
//    red: u8,
//    green: u8,
//    blue: u8
//}

//Tuple struct 
struct Color(u8, u8, u8);

//Used to create custom data types
pub fn run() {
//     let mut c = Color(255, 0, 0);

// //directly chage properties
//     println!("Color: {} {} {}",c.0, c.1, c.2);

//     c.0 = 200;
//     println!("Color: {} {} {}",c.0, c.1, c.2);

    let mut p = Person::new("John", "Doe");
    println!("Person {} {}",p.first_name, p.last_name);
    println!("{}", p.full_name());

    p.set_last_name("Williams");
    println!("{}", p.full_name());

    println!("{:?}", p.name_to_tuple());
}

struct Person {
    first_name: String, 
    last_name: String
}

impl Person {
    // Construct a person
    fn new(first: &str, last: &str) -> Person {
        Person {
            first_name: first.to_string(),
            last_name: last.to_string()
        }
    }

    fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    //set last name
    fn set_last_name(&mut self, last: &str) {
        self.last_name= last.to_string();
    }

    fn name_to_tuple(self) -> (String, String) {
        (self.first_name, self.last_name)
    }
}