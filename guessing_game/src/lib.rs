
#[cfg(test)]
mod tests {
    use super::*;
    use main;

    #[test]
    #[should_panic]
    fn exploration() {
        main::Guess::new(200);
    }

}
