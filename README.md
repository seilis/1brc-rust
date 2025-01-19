# 1BRC in Rust


## Measurements


### Original Java

Original Java reference program: 1m43.8s on my MacBook Pro.

```
./calculate_average_baseline.sh > official_answer.txt  100.98s user 4.03s system 101% cpu 1:43.80 total
```

### Naive Solution in Rust

- Ran with "release" and debug symbols on, but no other optimizations.

Time (1 run): 49.017 seconds on MBP.
Time (1 run): 1m42 seconds on Threadripper.

Comment: Amazing single threaded performance on the MBP. Could it be memory related?

What's next?

- perf/flamegraph
- Valgrind (dhat + massif)

Analysis:

The first time I tried to run DHAT, it was taking *forever*. So I ran with a smaller file which only had 10 million lines of input. While this won't give the correct final answer, it is enough to figure out what the process is doing generally.

First the memory analysis:

A bunch of memory is allocated at just one spot. Specifically 79,530,131 bytes at:
```
  │   ├── PP 1.1.1/2 {
  │   │     Total:     79,530,131 bytes (36.56%, 7,715.99/Minstr) in 10,000,000 blocks (100%, 970.2/Minstr), avg size 7.95 bytes, avg lifetime 421,154.58 instrs (0% of program duration)
  │   │     Max:       32 bytes in 4 blocks, avg size 8 bytes
  │   │     At t-gmax: 3,285 bytes (0%) in 413 blocks (98.1%), avg size 7.95 bytes
  │   │     At t-end:  0 bytes (0%) in 0 blocks (0%), avg size 0 bytes
  │   │     Reads:     239,059,289 bytes (14.81%, 23,193.47/Minstr), 3.01/byte
  │   │     Writes:    79,530,131 bytes (21.06%, 7,715.99/Minstr), 1/byte
  │   │     Allocated at {
  │   │       ^1: 0x48447A8: malloc (in /usr/lib/valgrind/vgpreload_dhat-amd64-linux.so)
  │   │       #2: 0x111C43: UnknownInlinedFun (alloc.rs:99)
  │   │       #3: 0x111C43: UnknownInlinedFun (alloc.rs:195)
  │   │       #4: 0x111C43: UnknownInlinedFun (alloc.rs:257)
  │   │       #5: 0x111C43: UnknownInlinedFun (raw_vec.rs:477)
  │   │       #6: 0x111C43: UnknownInlinedFun (raw_vec.rs:423)
  │   │       #7: 0x111C43: UnknownInlinedFun (raw_vec.rs:194)
  │   │       #8: 0x111C43: UnknownInlinedFun (mod.rs:803)
  │   │       #9: 0x111C43: to_vec<u8, alloc::alloc::Global> (slice.rs:159)
  │   │       #10: 0x111C43: to_vec<u8, alloc::alloc::Global> (slice.rs:108)
  │   │       #11: 0x111C43: to_vec_in<u8, alloc::alloc::Global> (slice.rs:502)
  │   │       #12: 0x111C43: to_vec<u8> (slice.rs:477)
  │   │       #13: 0x111C43: to_owned<u8> (slice.rs:885)
  │   │       #14: 0x111C43: to_owned (str.rs:211)
  │   │       #15: 0x111C43: from (string.rs:2880)
  │   │       #16: 0x111C43: to_string (string.rs:2795)
  │   │       #17: 0x111C43: process_raw_stations (main.rs:75)
  │   │       #18: 0x111C43: onebrc::main (main.rs:15)
  │   │       #19: 0x1148C2: call_once<fn(), ()> (function.rs:250)
  │   │       #20: 0x1148C2: std::sys::backtrace::__rust_begin_short_backtrace (backtrace.rs:154)
  │   │       #21: 0x1148B8: std::rt::lang_start::{{closure}} (rt.rs:195)
  │   │       #22: 0x130586: call_once<(), (dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe)> (function.rs:284)
  │   │       #23: 0x130586: do_call<&(dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe), i32> (panicking.rs:557)
  │   │       #24: 0x130586: try<i32, &(dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe)> (panicking.rs:520)
  │   │       #25: 0x130586: catch_unwind<&(dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe), i32> (panic.rs:358)
  │   │       #26: 0x130586: {closure#1} (rt.rs:174)
  │   │       #27: 0x130586: do_call<std::rt::lang_start_internal::{closure_env#1}, isize> (panicking.rs:557)
  │   │       #28: 0x130586: try<isize, std::rt::lang_start_internal::{closure_env#1}> (panicking.rs:520)
  │   │       #29: 0x130586: catch_unwind<std::rt::lang_start_internal::{closure_env#1}, isize> (panic.rs:358)
  │   │       #30: 0x130586: std::rt::lang_start_internal (rt.rs:174)
  │   │       #31: 0x11241B: main (in /home/ags/projects/1brc-aaron/1brc-rust/target/release/onebrc)
  │   │     }
```

Normally 79MB isn't a lot of memory, but it's all at one spot in our code. Interestingly, the `Total` line also points out that this is only 36.6% of the bytes allocated but 100% of the blocks. That's worth looking into. Interestingly the output doesn't appear to include any of the memory that must be used in `std::fs::read_to_string()`. We know that value has to be there somewhere, so it's worth seeing if we can find it

Let's see what `strace` has to say:
```
...
openat(AT_FDCWD, "measurements_10000000.txt", O_RDONLY|O_CLOEXEC) = 3
statx(3, "", AT_STATX_SYNC_AS_STAT|AT_EMPTY_PATH, STATX_ALL, {stx_mask=STATX_ALL|STATX_MNT_ID, stx_attributes=0, stx_mode=S_IFREG|0644, stx_size=137945363, ...}) = 0
mmap(NULL, 137949184, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x7d1696664000
read(3, "Sochi;28.9\nVirginia Beach;13.7\nJ"..., 137945363) = 137945363
read(3, "", 32)                         = 0
close(3)                                = 0
...
```

This shows that we used an `openat()` syscall to open the `measurements_10000000.txt` file, giving us file descriptor 3. We then called `statx()` which returned that the size of this file is 137,945,363 bytes (131 MB). Next, the `mmap()` syscall was used to create a private/anonymous memory region of 137949184 bytes. `PROT_READ|PROT_WRITE` means our process may both read and write to these pages. `MAP_PRIVATE` means that this memory map is private to our process (not shared) and `MAP_ANONYMOUS` means that this memory map is *not* backed by a file (something to revisit later, since we definitely have a file). The address for this mapping is `0x7d1696664000`.

We then have a `read()` call to our open file, putting 137945363 bytes into the memory location shown at the second argument. This is handled by the kernel, which would explain why DHAT doesn't see it because it's not a heap allocation.

So for memory, we have two major points to look at: the allocation created in `process_raw_stations()` that accounts for 36% of the bytes allocated and the `mmap` that appears to be used for reading the whole file.


Next let's look at the output of `cargo flamegraph`. See `flamegraph_01_initial.svg` for this. Wow, it's ugly. We've got a large stack for `onebrc::process_raw_stations` (as expected), but the stacks don't look like they were merged properly.

Let's try using the DWARF debug format in order to resolve the symbols to see if that helps.

```
CARGO_FLAMEGRAPH_PERF_ARGS="--call-graph dwarf" cargo flamegraph -- measurements_10000000.txt
```

And YES! This looks closer to what I would expect. At least we can see that `process_raw_stations()` is essentially all the work. Interestingly while `read_to_string()` does appear in the output, it looks very small. Maybe the mmap is very fast but this seems a bit suspicious to me. By default `perf` doesn't include kernel events so it's possible that the `read()` syscall is just missing. We can add a parameter to our flamegraph call to see what happens.

After a bit of playing around with parameters, it still didn't like my inputs. Since I know perf can do this, I'll circle back to see if I can enhance it.

For now, we can see the following top-level:

- `read_to_string()` - 1.5%
- `iter::Lines::next()` - 18.88%
- `iter::Split::next()` - 13.35%
- `ToString` - 7.52%
- `core::str::<impl str>::parse` (for f64) - 16.64%
- `Station::add_measurement()` - 0.79%
- `HashMap::entry()` - 39.48%

Again, this is just a data read out from a single run. We're not exactly quantifying how long each of these steps takes, just getting a ballpark figure. When we get down to smaller timings, we'll switch to a real benchmarking strategy but for now, we don't really care too much about the error bars of our measurements because they're much smaller than the total runtime of our program.

If we look at the above, the main time takers can be discussed in groups:

- Reading the file (`read_to_string`) shows as a small portion here but I'm still very suspicious that we're not accounting for the `read` syscall time in our perf run.
- Splitting the data into lines and fields takes about 32% of our runtime (`Lines as Iterator::next()`, `Split as Iterator::next()`).
- Parsing a float from the data seems to take a fair bit of time at 16.64%.
- Adding the measurement to a station only takes a small amount of time at less than 1%.
- Getting the entry from a HashMap seems to be the largest amount of time at 39.48%.


### Notes on HW

- Threadripper machine has 64GB of 2133 MT/s RAM.
- Macbook Pro has 24GB of 410 GB/s memory bandwidth
