use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use rust_containers::list::fixed::Fixed;

fn fixed_iter_10(c: &mut Criterion) {
    let size_10: [usize; 10] = std::array::from_fn(|v| v);

    c.bench_function("iter 10", |b| b.iter_batched_ref(
        || Fixed::with_list(size_10),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next() {}
        },
        BatchSize::SmallInput
    ));

    c.bench_function("iter 10 reverse", |b| b.iter_batched_ref(
        || Fixed::with_list(size_10),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next_back() {}
        },
        BatchSize::SmallInput,
    ));

    /*
    c.bench_function("iter2 10", |b| b.iter_batched_ref(
        || Fixed::with_list(size_10),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next() {}
        },
        BatchSize::SmallInput
    ));

    c.bench_function("iter2 10 reverse", |b| b.iter_batched_ref(
        || Fixed::with_list(size_10),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next_back() {}
        },
        BatchSize::SmallInput,
    ));
    */
}

fn fixed_iter_100(c: &mut Criterion) {
    let size_100: [usize; 100] = std::array::from_fn(|v| v);

    c.bench_function("iter 100", |b| b.iter_batched_ref(
        || Fixed::with_list(size_100),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next() {}
        },
        BatchSize::SmallInput
    ));

    c.bench_function("iter 100 reverse", |b| b.iter_batched_ref(
        || Fixed::with_list(size_100),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next_back() {}
        },
        BatchSize::SmallInput,
    ));

    /*
    c.bench_function("iter2 100", |b| b.iter_batched_ref(
        || Fixed::with_list(size_100),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next() {}
        },
        BatchSize::SmallInput
    ));

    c.bench_function("iter2 100 reverse", |b| b.iter_batched_ref(
        || Fixed::with_list(size_100),
        |list| {
            let mut iter = list.iter();

            while let Some(_v) = iter.next_back() {}
        },
        BatchSize::SmallInput,
    ));
    */
}

criterion_group!(benches, fixed_iter_10, fixed_iter_100);
criterion_main!(benches);
