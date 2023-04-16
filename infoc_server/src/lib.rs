pub use microkv::MicroKV;
#[cfg(debug_assertions)]
const DB_NAME: &str = "microkv_debug.db";

#[cfg(not(debug_assertions))]
const DB_NAME: &str = "microkv.db";

pub fn microkv_open() -> Result<MicroKV, Box<dyn std::error::Error>> {
    Ok(
        MicroKV::open_with_base_path(DB_NAME, std::env::current_dir()?)
            .expect("Failed to create MicroKV On-disk database"),
    )
}
