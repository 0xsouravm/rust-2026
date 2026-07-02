# 08: Async / Tokio — Notes

---

## 1. The C10K Problem

- **Year 1999**: Dan Kegel asks *"How do you handle **10,000 simultaneous**
  clients on a single server?"*
- In 1999, most servers handled **hundreds** of connections well, **thousands**
  was a wall.
- The prevailing model was **one thread per connection** → expensive in memory
  and scheduling.
- **10,000 threads ≈ 80 GB** of stack memory alone (8 MB × 10K).
- The fundamental waste: threads are **sleeping 95 % of the time** waiting on
  I/O, you're paying the full memory cost for threads that do nothing.
- This is the soul of async: do not pay for what you don't use.

---

## 2. The Hidden Tax: OS Context Switching

```
Timeline with 1000 threads (thread-per-request):
───Thread1──►│switch│──Thread2──►│switch│──Thread3──►│switch│──...
              1-10μs              1-10μs              1-10μs
```

- Per context switch: **~1–10 µs** of pure overhead.
- The OS saves/restores program counter, stack pointer, 16+ registers, TLB
  entries — and **CPU cache state** (the really expensive part).
- A single L1/L2 cache miss costs **100–300 CPU cycles**.
- **Async tasks avoid OS context switches** they yield cooperatively, staying
  in user space.

---

## 3. Thread-per-Request vs. Event Loop

**Thread-per-request (one chef per customer):**

```
Chef 1: Takes order → stands at oven doing nothing → serves
Chef 2: Takes order → stands at oven doing nothing → serves
...
(Hire 10,000 chefs to serve 10,000 customers? 😬)
```

**Event loop (one chef, many orders):**

```
Chef: Takes order 1 → puts dish 1 in oven →
      Takes order 2 → puts dish 2 in oven →
      Takes order 3 → puts dish 3 in oven →
      Oven 1 beeps! → Plates dish 1 → serves →
      ...
(One chef, 10,000 orders, NO blocking, full throughput!)
```

- The event loop polls: *"Is any I/O ready?"* → runs the handler → moves on.
- This is **cooperative multitasking** — tasks voluntarily yield when waiting
  for I/O.

---

## 4. What is a `Future`? 

**Async (Future): the pizza analogy:**

```
You: "I'd like a pizza." → You get a TICKET
You: go sit down, read a book, talk to friends
"Number 42!" → You go get your pizza
(You did useful things while the pizza cooked)
```

- A `Future` is a value representing a computation that **hasn't happened yet**.
- Futures are **lazy**, they do **nothing** until you `.await` or `poll` them.
- The `Future` trait:

```rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),   // "Your pizza is ready!"
    Pending,    // "Still cooking, we'll call you"
}
```

- `Pin<&mut Self>` — guarantee that the future won't be moved in memory (it
  may contain self-referential data in the generated state machine).
- `Context` carries a `Waker` — a callback the future uses to tell the
  executor *"I'm ready to be polled again."*

---

## 5. `async` / `.await` Syntax

### 5.1 Marking a function as `async`

```rust
use tokio::time::{sleep, Duration};

// This function returns impl Future<Output = String>
// The compiler generates the state machine for you
async fn fetch_user_name(user_id: u64) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("User #{}", user_id)
}
```

The compiler roughly generates:

```rust
// enum FetchUserNameFuture {
//     Start      { user_id: u64 },
//     WaitingSleep { user_id: u64, sleep_future: SleepFuture },
//     Done,
// }
```

### 5.2 Calling an `async` function

```rust
#[tokio::main]
async fn main() {
    // This does NOT execute fetch_user_name yet!
    // It just creates a Future value.
    let future = fetch_user_name(42);

    // THIS is what drives it to completion:
    let name = future.await;        // polls the future until Ready

    println!("Got: {}", name);       // "Got: User #42"
}
```

- `#[tokio::main]` is a proc-macro that transforms your `async fn main()`
  into a sync `main()` that starts a Tokio runtime and blocks the current
  thread while driving your future.
- `impl Future<Output = String>` — the function returns some concrete
  compiler-generated type that implements `Future`.

### 5.3 Concurrency with `tokio::join!`

**Sequential (slow):**

```rust
let a = fetch_data(1, 200).await;   // waits 200ms
let b = fetch_data(2, 150).await;   // THEN waits 150ms
let c = fetch_data(3, 100).await;   // THEN waits 100ms
// Total: ~450ms
```

**Concurrent (fast):**

```rust
let (a, b, c) = tokio::join!(
    fetch_data(1, 200),
    fetch_data(2, 150),
    fetch_data(3, 100),
);
// Total: ~200ms — limited only by the SLOWEST future
```

- `join!` is **concurrent but not parallel**, all futures run on a single
  task, interleaving their `.await` points.
- For dynamic-sized lists use `tokio::join_all` (or `futures::join_all`).

### 5.4 Error handling with `?`

```rust
use tokio::net::TcpStream;
use std::io;

async fn connect_to_server(addr: &str) -> Result<TcpStream, io::Error> {
    let stream = TcpStream::connect(addr).await?;   // ? works inside async fn
    Ok(stream)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match fetch_user_profile("127.0.0.1:5432", 42).await {
        Ok(profile)  => println!("✅ {}", profile),
        Err(e)       => eprintln!("❌ Error: {}", e),
    }
    Ok(())
}
```

---

## 6. Tokio Runtime Internals

```
┌──────────────────────────────────────────────────────────────┐
│                     TOKIO RUNTIME                            │
│                                                              │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │    EXECUTOR     │    │           REACTOR               │ │
│  │  Thread Pool    │    │  mio (cross-platform I/O)       │ │
│  │  ┌──────────┐  │    │  epoll / kqueue / IOCP          │ │
│  │  │ Thread N │  │    │  Wakes tasks via Waker          │ │
│  │  └──────────┘  │    └─────────────────────────────────┘ │
│  └─────────────────┘                                        │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           WORK-STEALING SCHEDULER                    │   │
│  │  Thread 1: [Task A] [Task C] [Task F]                │   │
│  │  Thread 2: [Task B]  ← idle, steals from Thread 1    │   │
│  │  Thread 3: [Task D] [Task E]                         │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

- **Executor** — drives `Future`s by calling `poll()`, manages the thread pool.
- **Reactor** — interfaces with the OS (`epoll`/`kqueue`/`IOCP`), calls
  `Waker::wake()` when I/O is ready.
- **Work-stealing scheduler** — keeps every CPU core busy by letting idle
  threads steal from busy ones.

### 6.1 Multi-thread vs. current-thread

| | multi-thread (default) | current-thread |
|--|------------------------|----------------|
| Worker threads | One per logical core | 1 |
| Concurrency | ✅ | ✅ |
| Parallelism | ✅ across cores | ❌ |
| Use case | Production servers | Tests, CLIs, embedded |

### 6.2 Configuring the runtime

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    println!("Running on Tokio multi-thread with 4 workers");
}

// Or build it manually for fine-grained control:
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .max_blocking_threads(32)
        .thread_name("myapp-worker")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async_main());
}
```

- `enable_all()` enables both the **I/O reactor** and the **time driver**
  (for `sleep` and timers). Selectively enable with `enable_io()` /
  `enable_time()` for minimal runtimes.
- Libraries should **return Futures**; only binaries pick a runtime.

---

## 7. `tokio::spawn` — Fire and Remember 🚀

```rust
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

async fn process_order(order_id: u64) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("Order {} processed", order_id)
}

#[tokio::main]
async fn main() {
    let handle: JoinHandle<String> = tokio::spawn(async move {
        process_order(42).await
    });

    println!("Task spawned, doing other work...");
    sleep(Duration::from_millis(50)).await;

    match handle.await {
        Ok(result)                   => println!("✅ {}", result),
        Err(e) if e.is_panic()       => eprintln!("❌ Task panicked!"),
        Err(e) if e.is_cancelled()   => eprintln!("⚠️ Task was cancelled"),
        Err(e)                       => eprintln!("❌ Unknown error: {}", e),
    }
}
```

- `spawn` returns a `JoinHandle<T>` — your contract with the task.
- Dropping the `JoinHandle` **detaches** the task (silent fire-and-forget):
  errors and panics are discarded.
- `tokio::spawn` requires the task to be `Send + 'static` — use `move`
  closures and `Arc<T>` to share data.
- Cancel with `handle.abort()` — cooperative, takes effect at the next
  `.await`.

### 7.1 `join!` vs. `spawn`

| | `tokio::join!` | `tokio::spawn` |
|--|----------------|----------------|
| Scheduling | Same task | **New task** (any worker) |
| Parallelism | Concurrent only | **Potentially parallel** |
| Cancellation | Drops with parent | **Independent lifetime** |
| Use case | Fixed set, all needed | Background / independent work |

---

## 8. Concurrency vs. Parallelism

```
CONCURRENCY (structure)         PARALLELISM (execution)
Single Thread / Single Core     Multiple Cores
                               
Task A: ████░░░░████░░░░████   Core 1: ████████████ Task A
Task B: ░░░░████░░░░████░░░░   Core 2: ████████████ Task B
Task C: ░░░░░░░░░░░░░░░░░░████ Core 3: ████████████ Task C
                               
All make progress, only one    Truly simultaneous
runs at a time
```

| | Concurrency | Parallelism |
|--|-------------|-------------|
| Cores needed | 1 | 2+ |
| Best for | I/O-bound work | CPU-bound work |
| Tokio tool | `join!` / `.await` | `spawn` across threads |
| Risk | Blocking tasks | Data races (prevented by Rust!) |

> Rust statically prevents data races in safe code. The borrow checker and
> `Send`/`Sync` are your friends.

---

## 9. Common Async Pitfalls

### 9.1 Blocking the executor

```rust
// ❌ WRONG — blocks the entire executor thread!
async fn bad_crypto_work() {
    let _hash = sha256_of_huge_file();  // 500ms of pure CPU
}

// ❌ ALSO WRONG — std::fs blocks the OS thread
async fn bad_file_read() -> String {
    std::fs::read_to_string("huge_file.txt").unwrap()
}

// ✅ CORRECT — spawn_blocking for CPU / blocking I/O
async fn good_crypto_work() {
    let result = tokio::task::spawn_blocking(|| {
        sha256_of_huge_file()
    })
    .await
    .expect("blocking task panicked");

    println!("Hash: {:?}", result);
}

// ✅ CORRECT — tokio::fs for async file I/O
async fn good_file_read() -> String {
    tokio::fs::read_to_string("huge_file.txt").await.unwrap()
}
```

Rule of thumb: **anything that takes more than ~100 µs of CPU time without
an `.await` point** should be wrapped in `spawn_blocking`. Use
`tokio::fs` / `tokio::net` instead of `std::fs` / `std::net`.

The blocking pool defaults to 512 threads, configurable via
`RuntimeBuilder::max_blocking_threads()`.

### 9.2 Forgetting `.await`

```rust
// ❌ WRONG — Future created and dropped, never runs
async fn send_notification(user_id: u64) {
    send_email(user_id);           // missing .await
}

// ✅ CORRECT
async fn send_notification_fixed(user_id: u64) {
    send_email(user_id).await;     // actually runs
}
```

Always read compiler warnings: *"unused value of type impl Future that
must be used"* — that means the computation will never run.

### 9.3 Holding a `Mutex` across `.await`

```rust
// ❌ WRONG — std::sync::Mutex + .await = potential deadlock
async fn bad_mutex_usage(shared: Arc<std::sync::Mutex<Vec<u64>>>) {
    let mut guard = shared.lock().unwrap();
    do_some_io().await;            // guard held across .await — 💥
    guard.push(42);
}

// ✅ Option 1: release the lock before .await
async fn good_mutex_short(shared: Arc<std::sync::Mutex<Vec<u64>>>) {
    {
        let mut guard = shared.lock().unwrap();
        guard.push(42);
    }                              // guard dropped here

    do_some_io().await;            // safe
}

// ✅ Option 2: use tokio::sync::Mutex — async-aware
async fn good_mutex_async(shared: Arc<tokio::sync::Mutex<Vec<u64>>>) {
    let mut guard = shared.lock().await;   // .await on lock acquisition
    do_some_io().await;
    guard.push(42);
}
```

Rule: use `std::sync::Mutex` for **short, fully synchronous** critical
sections. Use `tokio::sync::Mutex` whenever you must hold the lock
**across an `.await`**. Same rule applies to `RwLock`.

---

## 10. Work-Stealing in Detail

```
WORK-STEALING (KAAM CHOR) SCHEDULER — Step by Step

Initial State:
  Thread 1 queue: [Task A] [Task B] [Task C] [Task D]
  Thread 2 queue: []  ← idle!
  Thread 3 queue: [Task E]

Step 1: Thread 2 is idle — searches for work
  Thread 2: "Thread 1 has 4 tasks — I'll steal half!"

  Thread 1 queue: [Task A] [Task B]    ← Thread 1 keeps these
  Thread 2 queue: [Task C] [Task D]    ← Thread 2 steals these

Step 2: Task A hits an .await (waiting for network)
  → parked, Thread 1 moves on to Task B

Step 3: I/O ready! Waker wakes Task A
  Task A re-queued — could go to any thread's queue

Result: no thread sits idle. Work flows to wherever there's capacity.
```

- Per-thread local queues → no contention in the common case.
- Thieves take from the **far end** of a victim's queue → cache-friendly.
- In practice, near-linear scaling with CPU cores for I/O-bound workloads.

---

## 11. The One Rule

```
If it waits for the OS → async + .await
If it burns the CPU    → spawn_blocking
If it shares state     → Arc<tokio::sync::Mutex<T>>
```
