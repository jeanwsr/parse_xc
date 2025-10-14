mod parse;
use parse::parse;
mod libxc;
mod xc_helper;


fn main() {
    // println!("Hello, world!");
    // get a string from command line
    let args: Vec<String> = std::env::args().collect();
    let name = &args[1];
    // println!("parsing xc: {}", name);
    let final_results = parse(name);
    // println!("{}", final_results.formatted_output());
    final_results.summary();
}