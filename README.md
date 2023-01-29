run criterion benchmarks:

```
cargo bench
```

run custom test loop (for full results, requires `sudo` and [wbinv](https://github.com/batmac/wbinvd)):
```
cargo test --release
```

plot
```
cd results
python3 plot.py
```
