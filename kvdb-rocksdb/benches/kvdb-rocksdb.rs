#[macro_use]
extern crate criterion;
extern crate kvdb;
extern crate kvdb_rocksdb;
extern crate rand;
extern crate elapsed;
extern crate tempdir;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};
use kvdb_rocksdb::{DatabaseConfig, Database};
use kvdb::{DBTransaction, NumEntries};
use tempdir::TempDir;
use rand::{thread_rng, Rng};
use elapsed::measure_time;

fn randbytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0; n];
    thread_rng().fill(&mut buf[..]);
    buf
}

// Creates a database populated with sequential numbers as keys, and random bytes as values. Used
// only to provide other tests with a database with data in it to emulate a more realistic workload
// in the actual tests.
fn create_benchmark_db() -> String {
    use std::{fs, path};
    let size = 10_000_000;
    let batch_size = 1_000_000;
    let path = format!("./benches/dbs/bench-rocksdb-full-{}", size);
    let meta = fs::metadata(path::Path::new(&path));
    let db_cfg = DatabaseConfig::with_columns(Some(1));
    let db = Database::open(&db_cfg, &path.clone()).unwrap();

    let db_ok = if meta.is_err() {
        false
    } else {
        if let Ok(entries) = db.num_entries(0) {
            println!("DB has {} entries", entries);
            if entries >= size - batch_size && entries <= size + batch_size { true } else { false }
        } else {
            false
        }
    };

    if !db_ok {
        println!("Creating benchmark DB");
        let (elapsed, _) = measure_time(|| {
            let batches = size / batch_size;
            for b in 0..batches {
                println!("Batch: {}/{} – inserting indices {} –> {}", b, batches, b*batch_size, (b+1)*batch_size);
                let mut tr = DBTransaction::with_capacity(batch_size);
                for i in b*batch_size..(b+1)*batch_size {
                    let key = b*batch_size + i;
                    let slice = &unsafe { std::mem::transmute::<usize, [u8; 8]>(key) };
                    let v = randbytes(200); // TODO: need the distribution of payload sizes; match that to a distribution from `rand` and generate rand bytes accordingly?
                    tr.put(None, slice.as_ref(), &v);
                }
                db.write(tr).unwrap();
            }
        });
        println!("Created benchmark DB in {}", elapsed);
    }
    path
}

/// Create a new, empty DB and benchmark writes with 32 byte random keys and 200 byte long values.
fn write_to_empty_db(c: &mut Criterion) {
    let path = "./benches/dbs/bench-rocksdb-empty";
    let path2 = path.clone();
    c.bench(
        "empty DB, 32 byte keys",
        ParameterizedBenchmark::new(
            "payload size",
            move |b, payload_size| {
                let cfg = DatabaseConfig::with_columns(Some(1u32));
                let db = Database::open(&cfg, path).unwrap();
                let v = randbytes(*payload_size);
                b.iter(move || {
                    let mut batch = db.transaction();
                    let k = randbytes(32); // All ethereum keys are 32 byte
                    batch.put(None, &k, &v);
                    db.write(batch).unwrap();
                })
            },
            vec![ 10, 100, 1000, 10_000, 100_000 ],
        ).throughput(|payload_size| Throughput::Bytes(*payload_size as u32))
    );
    std::fs::remove_dir_all(std::path::Path::new(&path2)).unwrap();
}

/// Create a DB with 10 million 32-byte sequential keys with a 200 bytes value (also random).
/// Benchmark writes over different payload sizes (10 to 100 000 bytes).
fn write_to_ten_million_keys_db(c: &mut Criterion) {
    let db_path = create_benchmark_db();
    let db_path2 = db_path.clone();
    println!("DB path: {:?}", db_path);
    c.bench(
        "10Gbyte DB",
        ParameterizedBenchmark::new(
            "payload size",
            move |b, payload_size| {
                let db = Database::open(&DatabaseConfig::with_columns(Some(1)), &db_path).unwrap();
                let v = randbytes(*payload_size);
                b.iter(move || {
                    let mut tr = DBTransaction::with_capacity(1);
                    let k = randbytes(32);
                    tr.put(None, &k, &v);
                    db.write(tr).unwrap();
                })
            },
            vec![10, 100, 1000, 10_000, 100_000],
        ).throughput(|payload_size| Throughput::Bytes(*payload_size as u32))
    );
    std::fs::remove_dir_all(std::path::Path::new(&db_path2)).unwrap();
}

criterion_group!(benches,
    write_to_empty_db,
    write_to_ten_million_keys_db
);
criterion_main!(benches);
