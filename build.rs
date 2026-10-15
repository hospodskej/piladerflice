// The migrations are built into the program (sqlx::migrate!); rebuild when
// one is added or changed.
fn main() {
    println!("cargo:rerun-if-changed=db/migrations");
}
