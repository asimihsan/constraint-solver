use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fixed::types::extra::U64;
use fixed::FixedI128;
use fraction::GenericFraction;
use rust_decimal::Decimal;

fn fixed_add_one_third_to_third(c: &mut Criterion) {
    let fixed_1_3 = FixedI128::<U64>::from_num(1) / FixedI128::<U64>::from_num(3);
    let fixed_2_3 = FixedI128::<U64>::from_num(2) / FixedI128::<U64>::from_num(3);
    c.bench_function("Fixed  - 1/3 + 2/3", |b| {
        b.iter(|| black_box(fixed_1_3 + fixed_2_3))
    });
}

fn decimal_add_one_third_to_third(c: &mut Criterion) {
    let decimal_1_3 = Decimal::from(1) / Decimal::from(3);
    let decimal_2_3 = Decimal::from(2) / Decimal::from(3);

    c.bench_function("Decimal  - 1/3 + 2/3", |b| {
        b.iter(|| black_box(decimal_1_3 + decimal_2_3))
    });
}

fn fraction_u64_add_one_third_to_third(c: &mut Criterion) {
    let fraction_1_3: GenericFraction<u64> = GenericFraction::new(1u64, 3u64);
    let fraction_2_3: GenericFraction<u64> = GenericFraction::new(2u64, 3u64);

    c.bench_function("Fraction  - u64 - 1/3 + 2/3", |b| {
        b.iter(|| black_box(fraction_1_3 + fraction_2_3))
    });
}

fn fraction_i64_add_one_third_to_third(c: &mut Criterion) {
    let fraction_1_3: GenericFraction<i64> = GenericFraction::new(1i64, 3i64);
    let fraction_2_3: GenericFraction<i64> = GenericFraction::new(2i64, 3i64);

    c.bench_function("Fraction  - i64 - 1/3 + 2/3", |b| {
        b.iter(|| black_box(fraction_1_3 + fraction_2_3))
    });
}

fn float_add_one_third_to_third(c: &mut Criterion) {
    let f64_1_3: f64 = 1.0 / 3.0;
    let f64_2_3: f64 = 2.0 / 3.0;

    c.bench_function("Float - f64 - 1/3 + 2/3", |b| {
        b.iter(|| black_box(f64_1_3 + f64_2_3))
    });
}

criterion_group!(
    benches,
    fixed_add_one_third_to_third,
    decimal_add_one_third_to_third,
    fraction_u64_add_one_third_to_third,
    fraction_i64_add_one_third_to_third,
    float_add_one_third_to_third
);
criterion_main!(benches);
