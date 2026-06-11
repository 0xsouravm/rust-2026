// fn main() {
//     println!("Hello, world!");
// }


// Error handling in rust
// Testing
// File operations(context management)


// Error handling
// we handle errors during compile time

// Error handling in rust is done with 3 thinsg mostly
// 1. Panic!
// 2. Result
// 3. Option


//Panic
// so you are stuck in a bad situation from wher eyou can not recover
// Panic is used for the erros in system from which you can recover in your runtime
// [10, 20, 30]
// 4th index does not exist (index out of bound)

// you can use panic to print the error , unwinds the stack, exits or stop the program


// fn main() {
//     let age = 10;

//     if age <21{
//         panic!("You are not allowed in the bar");
//     } else {
//         println!("Welcome to the bar");
//     }

//     println!("The age of the consumer is {}", age);
// }



// Result
// Result is an enum which has two variants
// 1. Ok
// 2. Err

// In result enum 
// Ok mean susccess and it contains the result
// Err means it is a failure and it contains the error



// let's say I am diving x and y
// What if x is 20 and Y is 10 // reuslt cause 20 is divisuble by 10 and result will be 2
// What if x is 20 and y is 0 // In this case I can not divide 20 by 0 so i will have a zerio division error


// fn divide(x: i32, y: i32) {
// x/y
// }


// reuslt enum 
// enum Result<T, E> {
//  Ok(T),
//  Err(E)
// }


// T?
// we are using generic type T, A , B , C
// rust compiler enforces the rule that you should handle both result and error type
// 


// fn main(){
//     let good: Result<i32, String> = Ok(10);
//     //Here T is i32 and E is string
//     let bad: Result<i32, String> = Err(String::from("Something went wrong"));

//     match good {
//         Ok(value) => println!("The value is {}", value),
//         Err(e) => println!("Error: {}", e),
//     }

//     match bad {
//         Ok(value) => println!("The value is {}", value),
//         Err(e) => println!("Error: {}", e),
//     }

//     println!("The good result is {:?}", good);
//     println!("The bad result is {:?}", bad);
// }




// Option
// Option is an enum with field Some and None
// When you query something
// You might get the data  you might not
// Either something or nothing we can give error in this case 

// Option<T> {
//     Some(T),
//     None
// }


// fn main() {
//     let list = vec![1, 2, 3, 4, 5];

//     let index = 2;
//     let value = list.get(index);
//     match value {
//         Some(v) => println!("The value at index {} is {}", index, v),
//         None => println!("No value at index {}", index),
//     }

//     let index = 10;
//     let value = list.get(index);
//     match value {
//         Some(v) => println!("The value at index {} is {}", index, v),
//         None => println!("No value at index {}", index),
//     }
// }


// // ? operator
// // it is used to pass the error to the parent function 


// fn parent() -> Result<(), String> {

//     // what if we use match 
//     // let result = child();
//     // match result() {
//     //     Ok(_) => println!("Child function executed successfully"),
//     //     Err(e) => println!("Error in child function: {}", e),
//     // }


//     // let result = child();
//     // here the type of result is Result<(), String>
//     let result_1 = child_1()?;
//     // here the type of result_1 is ()
//     println!("The result of child function is");
//     let result_2 = child_2()?;
//     // here the type of result_2 is ()

//     println!("This line is not executed");
//     Ok(())
// }

// fn child_2() -> Result<(), String> {
//     Err(String::from("Something went wrong in child function"))
// }

// fn child_1() -> Result<(), String> {
//     Ok(())
// }


// fn main() {
//     match parent() {
//         Ok(_) => println!("Parent function executed successfully"),
//         Err(e) => println!("Error in parent function: {}", e),
//     }
// }

// unwrap and expect

// let's a variable with type Result and you apply unwrap on it or expect on it 
// for success it will just give you the value wrapped inside ok or some
// for error it will panic with your custom message


// fn main() {
//     let good: Result<i32, String> = Ok(10);

//     println!("The good result is {}", good.unwrap());
    
//     let bad: Result<i32, String> = Err(String::from("Something went wrong"));

//     println!("The bad result is {}", bad.unwrap_or(0));

//     // println!("The bad result is {}", bad.unwrap());
// }