mod mem64;
mod riscv;
mod error;

use clap::{Arg, App, crate_description, crate_authors, crate_version};
use xmas_elf::{ElfFile, header, program::{self, SegmentData}};
use mem64::{Endian, Physical, Protect, Config};
use riscv::{Fetch, Execute, Xlen};

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
    let xlen = match elf_file.header.pt1.class.as_class() {
        header::Class::ThirtyTwo => Xlen::X32,
        header::Class::SixtyFour => Xlen::X64,
        _ => panic!("unsupported xlen")
    }
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
    let mem = &mut mem as *mut _; // todo!
    let mut fetch = Fetch::new(unsafe { &*mem }, xlen); 
    let mut exec = Execute::new(unsafe { &mut *mem }, xlen);
    let mut pc = entry_addr;
    for _ in 0..10 {
        let (ins, mut pc_nxt) = fetch.next_instruction(pc).unwrap();
        println!("{:?}", ins);
        exec.execute(ins, pc, &mut pc_nxt).unwrap();
        exec.dump_regs();
        pc = pc_nxt;
    }
}
