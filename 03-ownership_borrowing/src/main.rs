// fn main() {
//     // println!("Hello, world!");

//     // first_rule();
//     // first_rule_break();
//     // second_rule();
//     // second_rule_breaks();
//     // third_rule();
//     third_rule_back();
//     ownership_with_integers();
//     // impl_string_copy();
//     // first_rule_break_test();
// }


// what is ownership?
// ownership is the rust's way of memory management. C, python , java they all have different memory management methods.
// c has malloc and free, python has garbage collector.
// so other language allow you to claim some memory and give you method to 
// clean up manually or  they clean it automatically.
// Rust's compiler tracks who owns what piece  of data and cleanup automaticlaly- during compile no runtime cost at all.

// when my program is compiled , the compiler makes sure all the values or all the data we have that should have owner,
// and when the owner or the task is done the data should be cleaned up.


// three rules of ownership
// 1. Each value has a single owner
fn first_rule(){
    // s is the owner of hello string
    // exactly one valriable owns one value
    let s = String::from("hello"); // String
    println!("{}", s);
}



fn first_rule_break(){
    // s1 is the owner of hello
    let s1 = String::from("hello");
    // here i am trying to assign value of s1 which is hello to s2
    // i am moving the ownership of hello from s1 to s2
    let s2 = s1;

    // s1 is no longer a valid variable

    // println!("{}", s1); 
    // hello is owned by s2 so s2 will print hello
    println!("{}", s2);
}

// rule 2: When owne goes out of the scope, the value will be dropped
fn second_rule(){
    {
        // this is another scope inside the scope of second rule function
        // create a value hello with an owner S
        let s = String::from("hello");
        // this will print hello cause s still owns hello
        println!("{}", s);
    }
}


// fn second_rule_breaks() {  
//     {
//         // this is another scope inside the scope of second rule function
//         // create a value hello with an owner S
//         let s = String::from("hello");
//         // this will print hello cause s still owns hello
//         println!("{}", s);
//     }
//     // here is s in the inner scope and this scope has no variable S
//     println!("{}", s); // this will give error because s is not valid here
// }


//rule 3: ownership can be transferred to another variable but only one variable can own a single value at a time
fn take_string(x: String){
    println!("ownership of {} is taken by take_ownership function", x);
}

// fn extra_func(y: String){
//     println!("ownership of {} is taken by extra_func function", y);
// }

// fn third_rule(){
//     // define a string hello and s is the owner of hello
//     let s = String::from("hello");
//     // here the value hello is moved to the func take_ownership and s is no nolonger the owner of hello
//     take_string(s);
//     // if i pass s again to any other function
//     extra_func(s); 
//     // println!("{}", s); 
// }


fn take_ownership_and_back(x: String) -> String {
    println!("ownership of {} is taken by take_ownership_and_back function", x);
    // here we are returning the value x which is hello and the ownership of hello is moved back to the caller

    // we create a string with coancatenation of x and world
    let y = x + " world";
    // if we modify the value x
    y
}

//hello was moved

fn third_rule_back(){
    let mut s = String::from("h");
    // here the value hello is moved to the func take_ownership_and_back and s is no nolonger the owner of hello
    if s.len() > 2 {
        take_string(s);
        // println!("value of s is {}", s);// should throw error
    } else {
        println!("{} is too short", s);
    }
    // s = take_ownership_and_back(s);
    // println!("value of x is {}", s);
    // after this there is value hello and s is the owner of hello again
    // println!("{}", s); 
}



// int is a data type which implements copy trait
// string was stored in heap memory without any specific size defined
// but inter is stored stack memory and easy to copy  cause we know the fixed size
// so copy types are those whoc implement copy trait, e.g. int, float, bool

// "hello"
// i8 ranges between -128 to 127
fn ownership_with_integers(){
    // let say we define a a value 5 and x is the owner
    let mut x =5;
    // ownership of 5 is moved Y - wrong
    //  the value 5 is copied to Y
    //now x and y are two different variable stored in stack
    // they eaach own a value which is 5
    let y = x;

    // we will increment x by 10
    x += 10;
    // x is no longer valid so x should not print
    println!("x: {}", x);
    println!("y: {}", y);
}


// how to copy a string so that we can pass it and still use it after passing to a function
// fn impl_string_copy(){
//     let s1 = String::from("hello");
//     // there are now two places in memory where is stored means hello is stored 2 times
//     // and s1 owns one hello and s2 owns another hello
//     let s2 = s1.clone();
//     take_string(s1);
//     // what if i need s1 here after this

//     // now we can copy the of s1 to  a variable s2 using clone method
//     // let s2 = s1.clone();
//     // now s1 is moved to take_ownership and s1 is no longer valid

//     println!(" trying to access s1 {}", s1); 
//     // println!(" trying to access s2 {}", s2); 
// }




// fn first_rule_break_test(){
//     // s1 is the owner of hello
//     let s1 = String::from("hello");
//     {
//         let s2 = s1;
//         println!("s2: {}", s2);
//     }
   
//     println!("s1: {}", s1);

//     // s1 is no longer a valid variable
//     // println!("{}", s1); 
//     // hello is owned by s2 so s2 will print hello
//     // println!("{}", s2);
// }


// In real rust code , you don't want to give ownership always to a function just that
// it can read or modify the values.

// Let's say hrit has car, now debi wants to use hrit's car
// hrit can give ownership of the car to debi 
// in that case hrit will not agree
// so to solve this hrit can simply lend hsi car to debi


// This is called borrowing in rust
// Rather than transfering the ownership of a value to a function we can simply
// let the function lend or borrow that value and use it.
// In this way the ownership is not transferred and function need is also satiisfied.


// how we can borrow a value in programming
// we can use a pointer that points to the memory location where the value is stored


// & is the symbol for reference/borrow in rust

// There are two types of references/borrows in rust
// 1. immutable reference : &t
// 2. mutable reference: &mut t

// so if i am letting debi take my car with a mutable contract then debi can modify the car
// but if i am letting debi take my car with an immutable contract then debi can only read the car but not modify it



// Three rules of borrowing
// 1: A reference must always point to a valid value

// fn main(){
//     // Declare a variable x and assign 5 to it
//     // x is the owner of 5
//     let x =5;
//     let y = &x;

//     println!("x: {}, y: {}", x, y);
// }


// fn main() {
//     let r;
//     {
//         let x=5;
//         r=&x; //r is pointing to memory location 0xhud8usg where x is stored
//         println!("x: {}, r: {}", x, r);
//     }// x is dropped here // now r is pointing to a memory location where x was stored
//     // this is called dangling reference
//     println!("r: {}", r);
//     // use after free error, 
// }


// 2: Many shared references or one mutable reference, at a single time, can never be both

// let's say hrit gives his car on borrow either with a mutable contract or immutable
// now let's say hrit gives his car to debi, arindam, digdarshan but on immutable contract
// so debi, arindam, digdarshan can ride the car but they can not modify it
// if hrit wants give his car for modification, then hrit can give to arindam


// one value can be shared with so many function to read it
// or can be shared with only function to modify it
// but never both at the same time

// x=5
// i can share &x with a , b , c , d ... number of varibales  as a immutable ref
// a= &x
// b= &x
// c= &x
// d= &x
// but i can only share one mutable referenc of x
// a = &mut x
// b = &mut x - wrong


// you can nnot do this also
// a = &x
//b = &mut x - wrong

// lexical and non lexical



// fn main() {
//     // created a variable x and asisgn value 42
//     let x =42;

//     // we create multiple iimutable borrow of value x
//     let r1 = &x; // immutable reference to x
//     let r2 = &x; // another immutable reference to x
//     let r3 = &x; // another immutable reference to x  

//     println!("r1: {}, r2: {}, r3: {}", r1, r2, r3);

// }

// fn main() {
//     let mut x = 42;

//     let r1 = &mut x; // mutable reference to x
//     let r2 = &mut x;

//     // bank balance example

//     // let mut balance = 1000;
//     // let r1 = &mut balance; // mutable reference to balance
//     // let r2 = &mut balance; // another mutable reference to balance

//     println!("r1: {}", r1);
//     println!("r2: {}", r2);
// }


// fn main() {
//     let mut x =5;
//     let r1 = &x; // immutable reference to x
//     let r2 = &mut x;
//     // println!("r1: {}, r2: {}", r1, r2);
// }


// non overlapping references are fine
// fn main() {
//     let mut x =5;

//     // rust compiler is smart, it tracks the last use of each refrence
//     // the last use of r1 is none so it considers that r1 is done

//     let r1 = &x; // immutable reference to x
//     let r2 = &x; // mutable reference to x

//     println!("r1: {}, r2: {}", r1, r2); // compiler understand that r 1 and r2 are done here

//     let r3 = &mut x; // mutable reference to x
//     *r3 += 10;
//     println!("r3: {}", r3);

//     println!("x: {}", x);

// }

// rust compiler is mahavir
// r1 is debi // take sthe car to drive // wed 
// r2 is arindam // take the car to modify it // fri

// fn main() {
//     let mut x =5;
//     let mut y = 10;

//     // rust compiler is smart, it tracks the last use of each refrence
//     // the last use of r1 is none so it considers that r1 is done

//     let r1 = &x; // immutable reference to x
//     let r2 = &x; // mutable reference to x

//     println!("r1: {}, r2: {}", r1, r2); // compiler understand that r 1 and r2 are done here

//     let mut r3 = &mut x; // mutable reference to x
//     *r3 += 10;
//     println!("r3: {}", r3);

//     r3 = &mut y; // mutable reference to y
//     *r3 += 20;
//     println!("r3: {}", r3);

//     println!("x: {}", x);
//     println!("y: {}", y);
// }


// 3: refrences don't have ownership

// fn main() {
//     // a reference is a pointer to a value it does not own anything, it just points to the
//     // owner's data. When owner goes out of the scope, nothing is freed. 
//     // cause it didn't own anything.
//     // if owner's data gets dropped bcz of going of the scope or anything
//     // then reference will also be invalid.

//     let s = String::from("hello");
//     { 
//         let r = &s;
//         println!("r: {}", r);
//     }

//     println!("s: {}", s);
// }



// 2nd june 2026
// doubt??


// 1: when you have to lend something to someone then the object should exist
// A reference must point to a valid value always
// 2: Multiple immutable references are valid, only one mutable reference is allowed, both of them can 
// never stay together
// 3: references don't have ownership


// borrowing through functions



// immumtable reference passing through a function

// fn main() {
//     let s = String::from("hello");
//     // here we are passing the reference of s to the function take_immutable_reference
//     take_immutable_reference(&s);
//     println!("value of s is {}", s);



//     let mut s2 = String::from("hello");
//     // we will pass s2 to a mutable function
//     take_mutable_reference(&mut s2);
//     println!("value of s2 is {}", s2);
// }

// fn take_immutable_reference(x: &String){
//     println!("taking immutable reference of {}", x);
//     println!("value of x is {}", x);
// }

// fn take_mutable_reference(x: &mut String){
//     println!("taking immutable reference of {}", x);

//     // we can modify value of x
//     x.push_str(" world");
//     println!("value of x is {}", x);
// }



fn main() {
    // so hrit has a car
    // hrit can give his car to others to use if they need
    // now govt says only the owner can driver the car 
    // or they have to make a rental contract with the person they want their car to drive

    let mut h_car = String::from("hrit's car");
    let h_car2 = h_car.clone();

    // so debi wants to use  hrit's car
    // but hrit is going out of city and don;t have time for contract
    // so hrit says take my car totally i will give you the ownership
    // debi_use_hrit_car(h_car);



    // debi wants to use hrit's car
    // but hrit also has need for his own car 
    // hrit will just buy a similar new car and give it to debi
    debi_use_hrit_car(h_car);


    // debi wants to use hrit's car
    // but hrit can not buy new and he has need of own car also 
    // as per govt rule hrit can lend car to debi

    // one is debi can just drive the car
    // another is debi is allowed to modify the car along with driving it

    // first case - immutable borrow -- a immutable contract
    debi_use_hrit_car_immut(&h_car);


    // second case - mutable broow - debi can modify the car
    // debi_use_hrit_car_mut(&mut h_car);

}


// debi wants to use hrit's car
fn debi_use_hrit_car(car: String){
    println!("debi is using {}", car);
}


fn debi_use_hrit_car_immut(car: &String){
    println!("debi is using {}", car);
}




