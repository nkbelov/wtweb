# Appendix: A *very* brief introduction to atomics

To implement new features, you will very likely have to interact with the internal code that uses atomic types and operations.
This section attempts to be a brief intro to the topic that will hopefully point you in the right direction for further research.

Atomics are the lowest level operations (besides direct assembly) that make it possible to synchronise different threads (or rather, CPU cores).

The general issue that makes atomics necessary in the first place is that different CPU cores are never in sync with one another, unless specifically instructed to do so.
While the intuitive model for a CPU would be that each core fetches data from the main memory, does something to it, and then puts it back, it couldn't be far from the truth. There are a lot of different hardware optimisations in play that disrupt this nice behaviour; these optimisations are guaranteed to produce no visible effects in a *single-threaded program* (that's why you never have to use atomics or mutexes or dispatch queues when working with just one thread), but for different threads, neither the compiler nor thr processor can predict just by looking at your code that you expect some operations to synchronisw with one another.

Due to these optimisations, each core generally operates on its own local copy of memory slices — these local copies are stored in what's called *the L1 cache*, and these caches are *separate* for each CPU core. Additionally, memory writes do not happen instantaneously; instead, they are also first cached in what's called *the write buffer*, and get flushed to the main memory only occasionally. Even then, the other cores do not by default immediately load whatever just got flushed by the first core.

All these mechanisms lead to different cores operating on their own view of the data, and only by issuing special instructions they can be forced to synchronise.
Git is a good analogy: different people work on different local copies until they deliberately fetch, sync and merge updates from different branches. This doesn't just happen automatically each time you write a character into a text file. Same for CPU cores.

While there are lots of different types of CPU optimisations (out-of-order execution, delayed writes etc.), what actually happens on the core doesn't matter to the program execution. They all boil down to one and only one *actually perceivable* effect: different operations seem to start happening in a wrong order when looked at from different threads' perspective (remember from above that this is guaranteed to never happen for single threaded programs). Consider:

```swift

var a = 10 // a is a global variable visible to every thread

// Thread 1
a += 10
a -= 10

// Thread 2
print(a)
```

— it is possible for the `print` statement in thread 2 to have printed `0`, even though from the perspective of thread 1 it should be impossible (we start with 10, add 10 to have 20, then subtract again to arrive at 10, but we never subtracted down to zero). Because, however, the CPU doesn't know that thread 2 is also interested in seeing these exact computations on `a` in a correct order, it never cares to also force the core running thread 2 to have the same memory state as the core for thread 1. Moreover, if there's a thread 3 that also does a `print(a)`, it could actually print `20` at the same time as thread 2 prints `0`. So, not only the apparent effects of operations on one thread can be wrong from the perspective of a different thread, *multiple threads are also not guaranteed to be "wrong" in the same way*. As long as the *final* result is correct, the CPU can choose to do either `10 -> 20 -> 10` or `10 -> 0 -> 10` as it pleases on one core, and the other cores can observe any of those possibilities.

## Synchronisation mechanisms
To force the different cores to synchronise, you use atomics. Atomics are implemented by having special machine instructions that perform a required operation (for instance, arithmetic or bitwise operations) *while simultaneously* instructing the CPU to *also* synchronise the chunks of memory that were affected with the other cores. These instructions differ from architecture to architecture, so instead of 

The usual synchronisation mechanisms, such as mutexes and/or dispatch queues, internally use special atomic instructions that force the relevant cores to synchronise their view of memory whenever you trigger the mutex or an operation on a queue starts executing. 

