#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {
       {
            eprint!("Error: ");
            eprintln!($($arg)*);
            std::process::exit(1)
       }
    };
}

pub fn format_size(n: u64) -> String {
    const UNITS: [char; 6] = ['K', 'M', 'G', 'T', 'P', 'E'];
    if n < 1024 {
        return format!("{} B ", n);
    }
    let bytes = n as f64;
    let i = (bytes.ln() / 1024_f64.ln()) as i32;
    format!(
        "{:.1} {}B",
        bytes / 1024_f64.powi(i),
        UNITS[(i - 1) as usize]
    )
}
