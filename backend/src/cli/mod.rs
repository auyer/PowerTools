mod args;
mod clean;
mod dump_sys;

pub use args::{Args, Operation};

pub fn do_op(op: &Operation) -> Result<(), ()> {
    match op {
        Operation::DumpSys => dump_sys::dump_sys_info(),
        Operation::Clean => clean::clean_up(),
    }
}
