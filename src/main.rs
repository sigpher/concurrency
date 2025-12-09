use std::{
    sync::{Arc, Mutex},
    thread,
};

fn main() {
    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for id in 0..100 {
        let builder = thread::Builder::new().name(id.to_string());
        let counter_clone = Arc::clone(&counter);
        let handle = builder
            .spawn(move || {
                let mut num = counter_clone.lock().unwrap();
                println!("thread {}", thread::current().name().unwrap());
                *num += 1;
            })
            .unwrap();
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
}
