use crate::helper::Helper::CLI;

mod chunker;
mod ingestors;
mod model;
mod tools;
mod vstore;
mod helper;


fn main() {
    let mut clargs = CLI::new();
    CLI::Parse_Args(&mut clargs);
    if clargs.debug{
        println!("{clargs:?}");
    }

    


}
