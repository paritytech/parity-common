#[macro_use]
extern crate criterion;
extern crate kvdb;
extern crate kvdb_lmdb;
extern crate rand;
extern crate elapsed;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};
use kvdb_lmdb::{DatabaseConfig, Database};
use tempdir::TempDir;
use rand::{thread_rng, Rng};
use kvdb::{DBTransaction, NumEntries};
use elapsed::measure_time;
use lmdb::{EnvironmentFlags, DatabaseFlags, WriteFlags};

fn randbytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0; n];
    thread_rng().fill(&mut buf[..]);
    buf
}

// Creates a database populated with sequential numbers as keys, and random bytes as values. Used
// only to provide other tests with a database with data in it to emulate a more realistic workload
// in the actual tests. The write settings here are highly unsafe and are not used in the actual
// tests.
fn create_benchmark_db() -> String {
    use std::{fs, path};
    let size = 10_000_000;
    let batch_size = 1_000_000;
    let path = format!("./benches/dbs/bench-lmdb-full-{}", size);
    let meta = fs::metadata(path::Path::new(&format!("{}/data.mdb", path)));
    // These settings are **NOT** ok for any other use than throw-away data
    let db_cfg = DatabaseConfig::new(1)
        .with_env_flags(EnvironmentFlags::NO_META_SYNC | EnvironmentFlags::NO_LOCK | EnvironmentFlags::NO_SYNC)
        .with_db_flags(DatabaseFlags::INTEGER_KEY)
        .with_write_flags(WriteFlags::APPEND);
    let db = Database::open(&db_cfg, &path.clone()).unwrap();

    let db_ok = if meta.is_err() {
        false
    } else {
        if let Ok(entries) = db.num_entries(0) {
            println!("DB has {} entries", entries);
            if entries == size { true } else { false }
        } else {
            false
        }
    };

    // Try to avoid re-creating the test DB for every test run.
    if !db_ok {
        println!("Creating benchmark DB");
        let (elapsed, _) = measure_time(|| {
            let batches = size / batch_size;
            for b in 0..batches {
                println!("Batch: {}/{} – inserting indices {} –> {}", b, batches, b*batch_size, (b+1)*batch_size);
                let mut tr = DBTransaction::with_capacity(batch_size);
                for i in b*batch_size..(b+1)*batch_size {
                    let v = randbytes(200); // TODO: need the distribution of payload sizes; match that to a distribution from `rand` and generate rand bytes accordingly?
                    tr.put(None, &i.to_ne_bytes(), &v);
                }
                db.write(tr).unwrap();
            }
        });
        println!("Created benchmark DB in {}", elapsed);
    }
    path
}

fn write_to_empty_db(c: &mut Criterion) {
    let path = "./benches/dbs/bench-lmdb-empty";
    let path2 = path.clone();
    c.bench(
        "empty DB, 32 byte keys",
        ParameterizedBenchmark::new(
            "payload size",
            move |b, payload_size| {
                let cfg = DatabaseConfig::new(1u32);
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

fn write_to_ten_million_keys_db(c: &mut Criterion) {
    let db_path = create_benchmark_db();
    let db_path2 = db_path.clone();
    println!("DB path: {:?}", db_path);
    c.bench(
      "10Gbyte DB",
      ParameterizedBenchmark::new(
        "payload size",
      move |b, payload_size| {
          let db = Database::open(&DatabaseConfig::new(1), &db_path).unwrap();
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
    // TODO: check that it is actually a directory we're removing
    std::fs::remove_dir_all(std::path::Path::new(&db_path2)).unwrap();
}

criterion_group!(benches,
//    write_to_empty_db,
    write_to_ten_million_keys_db,
);
criterion_main!(benches);
