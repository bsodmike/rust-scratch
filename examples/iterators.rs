use std::collections::HashMap;

fn main() {
    let collection1 = vec![1, 2, 3];
    let collection2 = vec![collection1];

    for r in collection2.iter().flat_map(|r| r.iter()) {
        println!("value: {}", r);
    }
    // notice that we did not move the contents of the collection above.
    let _ = collection2.iter().collect::<Vec<&Vec<i32>>>();

    // this is the naive way of accessing the inner iterator, which also causes a move due to
    // Rust's for-loop being syntactic sugar of the IntoIterator trait.
    for inner in collection2 {
        // contents are _moved_ into the iterator here.
        for items in inner {}
    }
    // collection2.iter();
    // ^
    // |__ we are now unable to iterate over it, due to the previous move.

    // the same applies for any other iterator; here's the same example with a HashMap:
    let inner = HashMap::from([("".to_owned(), 8)]);
    let collection = HashMap::<String, HashMap<String, i32>>::from([("a".to_string(), inner)]);

    for (k, v) in collection.iter().flat_map(|(_, r)| r.iter()) {
        println!("value: {}", v);
    }
}
