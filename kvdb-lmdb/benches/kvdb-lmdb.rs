#[macro_use]
extern crate criterion;
extern crate kvdb;
extern crate kvdb_lmdb;
extern crate rand;
extern crate elapsed;

use criterion::{Criterion, Benchmark, ParameterizedBenchmark, Throughput, black_box};
use kvdb_lmdb::{DatabaseConfig, Database};
use rand::{thread_rng, Rng, distributions::Uniform};
use kvdb::{DBTransaction, NumEntries};
use elapsed::measure_time;
use lmdb::{EnvironmentFlags, DatabaseFlags, WriteFlags};
use parking_lot::RwLock;
use std::{fs, path, thread, sync::Arc, sync::atomic::{AtomicBool, Ordering}};

fn randbytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0; n];
    thread_rng().fill(&mut buf[..]);
    buf
}

const DB_SIZE:usize = 10_000_000;

// Creates a database populated with sequential numbers as keys, and random bytes as values. Used
// only to provide other tests with a database with data in it to emulate a more realistic workload
// in the actual tests. The write settings here are highly unsafe and are not used in the actual
// tests.
fn create_benchmark_db() -> String {
    let batch_size = 1_000_000;
    let path = format!("./benches/dbs/bench-lmdb-full-{}", DB_SIZE);
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
            if entries == DB_SIZE { true } else { false }
        } else {
            false
        }
    };

    // Try to avoid re-creating the test DB for every test run.
    if !db_ok {
        println!("Creating benchmark DB");
        let (elapsed, _) = measure_time(|| {
            let batches = DB_SIZE / batch_size;
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
    c.bench(
      "write to non-empty DB",
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
    std::fs::remove_dir_all(std::path::Path::new(&db_path2)).unwrap();
}

fn read_random_keys(c: &mut Criterion) {
    let db_path = create_benchmark_db();
    c.bench(
        "random read from non-empty DB",
        Benchmark::new(
            "5k keys",
            move |b| {
                let db = Database::open(&DatabaseConfig::new(1), &db_path).unwrap();
                // Doing the sampling here, outside the measured code so that we request the same
                // keys for every iteration. This means that we're measuring the performance with
                // hot caches. When testing with random keys for every iteration results vary quite
                // a lot between benchmark runs, presumably because the OS fills up the page cache.
                let keys: Vec<usize> = thread_rng().sample_iter(&Uniform::from(0..DB_SIZE)).take(5000).collect();
                b.iter(move || {
                    for key in keys.iter() {
                        black_box(db.get(None, &key.to_ne_bytes()).unwrap());
                    }
                })
            },
        )
    );
}

fn read_random_keys_concurrently(c: &mut Criterion) {
    let db_path = create_benchmark_db();
    let db_cfg = DatabaseConfig::new(1);
    let db = Database::open(&db_cfg, &db_path).unwrap();
    let db = Arc::new(RwLock::new(db));
    let stop_thrs = AtomicBool::new(false);
    let stop_thrs = Arc::new(stop_thrs);
    for _i in 0..4 {
        let dbthr = db.clone();
        let stop_thr = stop_thrs.clone();
        thread::spawn(move || {
            let db = dbthr.read();
            let mut rng = thread_rng();
            loop {
                if stop_thr.load(Ordering::Relaxed) { break; }
                let key = rng.gen_range(0, DB_SIZE);
                black_box(db.get(None, &key.to_ne_bytes()).unwrap().unwrap());
            }
        });
    }
    let db_bm = db.clone();
    c.bench("random read from non-empty DB, concurrent",
        Benchmark::new("5k keys",move |b| {
                let db = db_bm.read();
                // Doing the sampling here, outside the measured code so that we request the same
                // keys for every iteration. This means that we're measuring the performance with
                // hot caches. When testing with random keys for every iteration results vary quite
                // a lot between benchmark runs, presumably because the OS fills up the page cache.
                let keys: Vec<usize> = thread_rng().sample_iter(&Uniform::from(0..DB_SIZE)).take(5000).collect();
                b.iter(move || {
                    for key in keys.iter() {
                        black_box(db.get(None, &key.to_ne_bytes()).unwrap());
                    }
                })
            }

        )
    );
    stop_thrs.store(true, Ordering::Relaxed);
}

fn read_sequential_keys(c: &mut Criterion) {
    let db_path = create_benchmark_db();
    c.bench("sequential read from non-empty DB, single thread",
        ParameterizedBenchmark::new(
        "#keys",
        move |b, nr_keys| {
            let db = Database::open(&DatabaseConfig::new(1), &db_path).unwrap();
            let mut rng = thread_rng();
            let start = rng.gen_range(0, DB_SIZE - nr_keys);
            b.iter(move || {
                for key in start..(start + nr_keys) {
                    black_box(db.get(None, &key.to_ne_bytes()).unwrap());
                }
            })
        },
        vec![10, 100, 1000],
    ));
}

fn read_sequential_keys_concurrently(c: &mut Criterion) {
    let db_path = create_benchmark_db();
    let db_cfg = DatabaseConfig::new(1);
    let db = Database::open(&db_cfg, &db_path).unwrap();
    let db = Arc::new(RwLock::new(db));
    let stop_thrs = AtomicBool::new(false);
    let stop_thrs = Arc::new(stop_thrs);
    for _i in 0..4 {
        let dbthr = db.clone();
        let stop_thr = stop_thrs.clone();
        thread::spawn(move || {
            let db = dbthr.read();
            let mut rng = thread_rng();
            loop {
                if stop_thr.load(Ordering::Relaxed) { break; }
                let key = rng.gen_range(0, DB_SIZE);
                black_box(db.get(None, &key.to_ne_bytes()).unwrap().unwrap());
            }
        });
    }

    let db_bm = db.clone();
    c.bench("sequential read from non-empty DB, 5 threads",
        ParameterizedBenchmark::new(
        "#keys",
        move |b, nr_keys| {
            let db = db_bm.read();
            let mut rng = thread_rng();
            let start = rng.gen_range(0, DB_SIZE - nr_keys);
            b.iter(move || {
                for key in start..(start + nr_keys) {
                    black_box(db.get(None, &key.to_ne_bytes()).unwrap());
                }
            })
        },
        vec![10, 100, 1000],
    ));
    stop_thrs.store(true, Ordering::Relaxed);
}

criterion_group!(benches,
    write_to_empty_db,
    read_random_keys_concurrently,
    read_random_keys,
    read_sequential_keys_concurrently,
    read_sequential_keys,
    write_to_ten_million_keys_db, // this one last as it writes to the db
);
criterion_main!(benches);
