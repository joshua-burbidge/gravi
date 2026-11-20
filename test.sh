cargo test --test orbital_test test_earth_completes_one_orbit -- --nocapture
# nocapture = prints output

# Run one bench group
cargo bench --bench simple-bench
