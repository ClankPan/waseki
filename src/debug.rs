// #[macro_export]
// macro_rules! debug {
//     ($($a:expr),* $(,)*) => {
//         #[cfg(debug_assertions)]
//         eprintln!(concat!($("| ", stringify!($a), "={:?} "),* "|"), $(&$a),*);
//     };
// }
