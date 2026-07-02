use tokio::time::{sleep, Duration};

// #[tokio::main(flavor = "multi_thread", worker_threads = 4)]
// async fn main() {
//     // Explicitly 4 worker threads
//     println!("Running on Tokio multi-thread with 4 workers");
// }

// Or manual runtime creation for fine-grained control:
// fn main() {
//     let rt = tokio::runtime::Builder::new_multi_thread()
//         .worker_threads(8)
//         .max_blocking_threads(32)
//         .thread_name("myapp-worker")
//         .enable_all()
//         .build()
//         .unwrap();
    
//     rt.block_on(async_main());
// }

// async fn async_main() {
//     println!("Running custom runtime!");
// }


async fn fetch_data(item: u32, delay: Duration) -> u32 {
    sleep(delay).await;
    println!("Fetched data for item {}", item);
    item
}
use tokio::task::spawn_blocking;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {


    // // Example async tasks using Tokio Join
    // let results = tokio::join!(
    //     // Tasks 
    //     fetch_data(1, Duration::from_secs(2)), // T1
    //     fetch_data(2, Duration::from_secs(5)), // T2
    //     fetch_data(3, Duration::from_secs(9)), // T3
    //     fetch_data(4, Duration::from_secs(1)) // T4
    // );
    // println!("Results: {:?}", results);


    let results3 = tokio::select! {
        // Tasks — select returns the first one to complete
        res1 = fetch_data(0, Duration::from_secs(2)) => res1,
        res2 = fetch_data(1, Duration::from_secs(1)) => res2
    };

    // println!("Results3: {:?}", results3);


    // Multiple async tasks using Tokio Spawn
    let handles = (1..=10).map(|i| {
        tokio::spawn(fetch_data(i, Duration::from_secs(1)))
    }).collect::<Vec<_>>();
    
    for handle in handles {
        match handle.await {
            Ok(result) => println!("Task completed with result: {}", result),
            Err(e) => eprintln!("Task failed: {:?}", e),
        }
    }

    // Arc - Address to the heap shared across threads
    spawn_blocking(|| async {
        let data = fetch_data(1, Duration::from_secs(2)).await;
        println!("Data from blocking task: {}", data);
    });

}