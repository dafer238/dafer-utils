use polars::prelude::*;

fn read_csv(path: &str) -> Result<LazyFrame, PolarsError> {
    LazyCsvReader::new(PlPath::new(path))
        .finish()
        .map_err(|err| {
            eprintln!("Failed to read CSV at {}: {}", path, err);
            err
        })
}

fn main() {
    let filepath = "./data/Iris.csv";

    let mut lf = read_csv(filepath).unwrap();
    lf = lf.with_columns([
        // (col("PetalLengthCm") * lit(2)).alias("PetalLenghtCm times 2"),
        (col("PetalWidthCm").pow(10))
            .round(4, RoundMode::HalfToEven)
            .alias("PetalWidthCm pow 2"),
        // (col("SepalWidthCm") / lit(2)).alias("SepalWidthCm by 2"),
        // (col("SepalLengthCm") + lit(1)).alias("LengthWidthCm plus 1"),
    ]);
    let _ = dbg!(lf.collect());
}
