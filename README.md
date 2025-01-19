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
