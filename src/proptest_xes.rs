#[cfg(test)]
mod tests {
    use crate::io::xes::XESReader;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_xes_parsing_is_stable(s in r"\PC*") {
            let reader = XESReader::new();
            let _ = reader.parse_str(&s);
        }
    }
}
