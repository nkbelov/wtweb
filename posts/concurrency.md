# A comprehensive guide to Swift concurrency

TOC:
- Threads and examples
- Manual synchronisation primitives for threads
  - idk perhaps explain what critical sections are
- *ATOMICS* and show them before all this async/await stuff
- Swift ARC is atomic
- GCD and its internals (queue, continuation, "wakeup")
- async/await syntax as a closure rewrite
- async/await is not a panacea it's for waiting and doesn't help with acceleration accelerate

## What is concurrency and why you would need it

Many comprehensive texts provide some sort of 

It's important to realise [ ] because if you look at it, it actually just seems like concurrency/multithreading etc. is just a huge pain that only exists to make programmers sad. Unfortunately, why it indeed raises the complexity of a program dramatically, it's important to understand that in the end, parallel processing is a net benefit that was introduced because there's no better way to solve the problems it tries to solve.

[ processors ]
[ overclocking ]
[ latency internet etc. ]
[ more cores as solution ]
[ maybe have a look at GPUs ]

## Concurrency vs. parallelism vs. multithreading

[ Important note: all parallelism is concurrency. even single-core can be concurrent. it's concurrency that's problematic, not parallelism ]

## Async vs. acceleration

At this point, it's important to realise one crucial thing. Using async/await or any related concurrency features *does not mean accelerating your code*. Async/await only really helps in contexts where one piece of your code has to wait for something, and there's other stuff to do *other than* just waiting. If you're literally bound by this waiting

In other words, async/await helps you swap out a waiting task to perform some other work in the meantime. *If there's nothing else to do, your program will equally be waiting for that single task's completion as if there wasn't any async/await at all*.

Async/await is not a cost-free abstraction that the processor "just does to make your program run faster". What we have discussed above should have clearly shown that there's some performance overhead to enqueueing tasks, sorting through them and 

To be more [], here are examples of tasks that make vs. not make sense to execute concurrently:

#### Non-parallelisable tasks:
- Encryption
- Compression/decompression


#### Good example: streaming/large download

[ Talk about recurring callbacks for receiving a large file ]