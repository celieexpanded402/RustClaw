#![feature(test)]

extern crate criterion;
extern crate your_project_name; // Replace with actual project name or relative path to the module that contains cosine_similarity
use criterion::Criterion;

fn cosine_similarity_bench(c: &mut Criterion) {
    let vector1 = vec![0.5, -0.5, 1.0];
    let vector2 = vec![-0.7, 1.0, 0.3];
    c.bench_function("cosine_similarity", |b| b.iter(|| your_project_name::cosine_similarity(&vector1, &vector2)));
}

criterion_group!(benches, cosine_similarity_bench);
criterion_main!(benches);