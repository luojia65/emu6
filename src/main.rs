mod mmu64;
mod riscv64;

use clap::{Arg, App, crate_description, crate_authors, crate_version};
use xmas_elf::{ElfFile, header, program::{self, SegmentData}};
use mmu64::{Endian, Physical, Protect, Config};
use std::sync::Arc;

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

    let elf_file_name = matches.value_of("target programs").unwrap();
    let elf_buf = std::fs::read(elf_file_name)
        .expect("read target program");
    let elf_file = ElfFile::new(&elf_buf).expect("open the elf buffer");
    match elf_file.header.pt2.type_().as_type() {
        header::Type::Executable => {
            // trace!("this is an elf executable!");
        },
        fallback => {
            panic!("unsupported elf type: {:?}!", fallback);
        }
    }
    let entry_addr = matches.value_of("pc")
        .map(|s| u64::from_str_radix(s, 16).expect("convert input pc value"))
        .unwrap_or(elf_file.header.pt2.entry_point());
    println!("Entry point: 0x{:016X}", entry_addr);
    let mut mem = Physical::new();
    let endian = match elf_file.header.pt1.data.as_data() {
        header::Data::BigEndian => Endian::Big,
        header::Data::LittleEndian => Endian::Little,
        _ => panic!("invalid endian")
    };
    for program_header in elf_file.program_iter() {
        if program_header.get_type() != Ok(program::Type::Load) {
            continue;
        }
        let vaddr = program_header.virtual_addr();
        let mem_size = program_header.mem_size();
        let data = match program_header.get_data(&elf_file).expect("get program data") {
            SegmentData::Undefined(data) => data,
            _ => unreachable!(),
        };
        let mut protect = Protect::empty();
        if program_header.flags().is_execute() {
            protect |= Protect::EXECUTE;
        }
        if program_header.flags().is_read() {
            protect |= Protect::READ;
        }
        if program_header.flags().is_write() {
            protect |= Protect::WRITE;
        }
        let config = Config {
            range: vaddr..(vaddr + mem_size),
            protect,
            endian,
        };
        if protect.contains(Protect::WRITE) {
            mem.push_owned(config, data.to_vec());
        } else {
            mem.push_slice(config, data);
        }
    }
    // println!("Memory: {:?}", mem);
    let mem = Arc::new(mem);
    let mut fetch = riscv64::Fetch { inner: Arc::clone(&mem), pc: entry_addr };
    println!("{:?}", fetch.next_instruction().unwrap());
}