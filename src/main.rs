mod mmu64;
mod riscv64;

use clap::{Arg, App, crate_description, crate_authors, crate_version};
use xmas_elf::ElfFile;

fn main() {
    let matches = App::new("emu6")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("debug")
            .short("d")
            .help("Enable an interactive debug console"))
        .arg(Arg::with_name("pc")
            .long("pc")
            .help("When using single executable, override ELF entry point")
            .takes_value(true))
        .arg(Arg::with_name("target programs")
            .help("Input programs; typically one or multiple ELF files")
            .required(true)
            .index(1))
        .get_matches();
    println!("pc={:?}", matches.value_of("pc"));

    let elf_file_name = matches.value_of("target programs");
    // let buf = 
    // let elf_file = ElfFile::new()
    let mut mem = mmu64::Physical::new();
}
