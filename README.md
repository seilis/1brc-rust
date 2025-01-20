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

### Stop allocating needlessly

Normally, I look for low-hanging fruit and that's often in the largest frame. When I look at the `HashMap::entry()` block one thing immediately strikes me: inside the `rustc_entry` frame, there's a call to `drop_in_place<alloc::string::String>` and it takes 4.76% of the total runtime! That's happening because we're passing in a `String` to the `HashMap::entry()` call but most of the time the string exists.

This makes sense if we consider that the original 1brc problem states that there are max 10,000 stations but there are 1 billion rows and we're using this string to hold the station name. So we only need that string 0.001% of the time. Not a good trade-off.

There are a couple of ways we could tackle this problem:

- Keep using the `entry()` method with a `Cow`, which lets us avoid extra allocations in most places.
- Use the `get()` method, which can take a reference to the key and only create the key when needed.

While the first one is fancy, we can try the `get()` first, since it's simple.

We can replace our original entry wall with this:
```
        let station_maybe = stations.get_mut(name);

        let station = if station_maybe.is_some() {
            station_maybe.unwrap()
        } else {
            drop(station_maybe);
            stations.insert(name.to_string(), Station::new(value));
            stations.get_mut(name).unwrap()
        };
```

Perhaps it's not the most elegant looking code but it has the benefit of having a short path (only one hashtable lookup) when the key exists although we do it twice when the key doesn't exist.

What do the timings look like now?

Time (1 run): 39.7 seconds on MBP. (19% runtime reduction)
Time (1 run): 1m11s on Threadripper. (30% runtime reduction)

Woah. New flamegraph is `flamegraph_03_stop_allocating_needlessly.svg`. I only really expected that to improve performance by about 7.5%+4.76%=12%, which is respectable but we got a whopping 19% reduction in runtime on the MBP and 30% on the Threadripper! First of all, we can collect those 12% savings from the memory saved, but it looks like for `String`, the `HashMap::entry()` method is also slower than `get_mut()`.

It's worth noting that `get_mut()` is still 29.6% of our runtime so it's worth continuing to dig into this, but it's a great first step.


### Faster hash

Okay so our `HashMap` is still too slow for our goals. If we dig into the `HashMap::get_inner_mut` calls from our previous run, we can see that `hashbrown::map::make_hash` accounts for about 12% of the total runtime, while `hashbrow::raw::RawTableInner::find_inner` is about 16% and that includes about 10.45% that is doing a string comparison during the find.

There are faster hashes that are available, including `FxHashmap`. Thankfully this is a drop-in replacement for `HashMap` so we can give it a go:

Time (1 run): 32.6 seconds on MBP.
Time (1 run): 1m5s on Threadripper.

### Faster hash, round 2

Okay so we got a decent further improvement in the runtime for our `HashMap` with FxHash but it's *still* taking 25% of the runtime and the time is still being spent in a string equivalence lookup. We need to be able to reduce that further.

Now we have to be a bit careful, but the `make_hash` function is now only 2.43% of our runtime. If the time taken for the lookup is mostly comparisons, perhaps we can use a hash value itself as the key to our map. This would have the negative effect that we would be computing a double hash (one for the input string and another for the hash of the hash) but it might be an acceptable price to pay. We could also try using the hash as the key for a BTreeMap, which is not O(1) but should require only a small number of comparisons before returning the value. In order to be able to do this, we'll need to store the full string inside the Station struct so that we can access the actual names later.

Time (1 run): 31 seconds on MBP. (37% runtime reduction)
Time (1 run): 1m1 on Threadripper. (40% runtime reduction)

`docs/images/flamegraph_05_fxhash_with_hash_key.svg`

We're now at a similar reduction in runtime for the two platforms, but the M4 CPU exhibits much better single-threaded performance than the Threadripper.

At this point the `hash()` function is taking 2.7% of runtime and `HashMap::get_mut()` is taking 9%. There may be more gains to be made here later, but this is significantly reduced from our previous attempts. Further enhancements here won't (yet) show large runtime reductions since they're being dwarfed by the other portions.

### BTreeMap

I also tried with a BTreeMap, but the Threadripper went back up to 1m24s. Looks like double hashing is much faster.

### Memmap2

Tried with `memmap2` as a memory map and it does indeed remove the `read()` call that we saw earlier, but it turns out this was fast because we only saved a tiny bit of time. Probably the kernel can mostly just set up pages as copy-on-write (or similar) for the `read()` call and so it's fairly fast.

```
openat(AT_FDCWD, "measurements.txt", O_RDONLY|O_CLOEXEC) = 3
statx(3, "", AT_STATX_SYNC_AS_STAT|AT_EMPTY_PATH, STATX_ALL, {stx_mask=STATX_ALL|STATX_MNT_ID, stx_attributes=0, stx_mode=S_IFREG|0644, stx_size=13795310865, ...}) = 0
mmap(NULL, 13795310865, PROT_READ, MAP_SHARED, 3, 0) = 0x7c4667600000
close(3)
```

We do see a small amount of time spent in `from_utf8`, about 2.7%. We can remove that time by using the "unchecked" version. Normally I would say that this isn't worth it, but we do want to go as fast as possible and it's a controlled scenario here.

Time (1 run): 50 seconds on MBP.
Time (1 run): 56 seconds on Threadripper.

🤯 we've strayed off the happy path, at least for the MBP. So we'll put the file-reading commands into a compile-time switch based on OS.


### Notes on HW

- Threadripper machine has 64GB of 2133 MT/s RAM.
- Macbook Pro has 24GB of 410 GB/s memory bandwidth
