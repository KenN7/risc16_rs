use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fmt::Write as FmtWrite;

enum Archtype {
    IS0,
    IS1,
    IS2,
}

struct Risc16 {
    registers: [i16; 8],
    pc: usize,
    ram: [i16; 256],
    instr_count: u32,
    max_instr: u32,
    labels: HashMap<String, usize>,
    arch: Archtype,
    buffer: String,
}

impl Risc16 {
    fn build(arch: Archtype) -> Risc16 {
        Risc16 {
            registers: [0; 8],
            pc: 0,
            ram: [0; 256],
            instr_count: 0,
            max_instr: 10000,
            labels: HashMap::new(),
            arch,
            buffer: String::new(),
        }
    }

    fn execute(&mut self, rom: &[String], labels: HashMap<String, usize>) {
        self.labels = labels;
        for _instr in 0..self.max_instr {
            match self.execute_instr(rom.get(self.pc).expect("Reaching end of ROM, missing HALT")) {
                Some(_result) => (),
                None => break,
            }

            self.instr_count += 1;
            self.pc += 1;
            self.registers[0] = 0;
        }
    }

    fn execute_instr(&mut self, full_instr: &str) -> Option<bool> {
        lazy_static! {
            static ref RE_SPC: Regex = Regex::new(r"\s+").unwrap();
        }
        let vec_instr: Vec<&str> = RE_SPC.splitn(full_instr, 2).collect();
        let instr = vec_instr.get(0).unwrap();
        let args = vec_instr.get(1).unwrap_or(&"");
        // self.display_state(false);
        println!("{}({})", instr, args);
        writeln!(self.buffer, "{}({})", instr, args).unwrap();
        

        match *instr {
            "nop" => self.nop(args),
            "halt" => self.halt(args),
            "reset" => self.reset(args),
            "add" => self.add(self.process_args_3(args, instr)),
            "addi" => self.addi(self.process_args_2i(args, instr)),
            "nand" => self.nand(self.process_args_3(args, instr)),
            "movi" => self.movi(self.process_args_1i(args, instr)),
            "lui" => self.lui(self.process_args_1i(args, instr)),
            "lw" => self.lw(self.process_args_2i(args, instr)),
            "sw" => self.sw(self.process_args_2i(args, instr)),
            "beq" => self.beq(self.process_args_2i(args, instr)),
            "jalr" => self.jalr(self.process_args_2(args, instr)),
            _ => {
                println!("Error: Instr not know: {}", instr);
                writeln!(self.buffer,"Error: Instr not know: {}", instr).unwrap();
                None
            }
        }
    }

    fn display_state(&mut self, full: bool) {
        print!("PC: {}, Instr. count: {}", self.pc, self.instr_count);
        write!(self.buffer, "PC: {}, Instr. count: {}", self.pc, self.instr_count).unwrap();
        println!(", regs: {:?}", self.registers);
        writeln!(self.buffer,", regs: {:?}", self.registers).unwrap();
        if full {
            println!("ram: {:?}", self.ram);
        }
    }

    fn process_args_3(&self, args: &str, _instr: &str) -> Vec<usize> {
        let vec_arg = args
            .split(',')
            .map(|x| x.parse::<usize>().expect("Cannot parse int"))
            .collect();
        vec_arg
    }

    fn process_args_2i(&self, args: &str, _instr: &str) -> (usize, usize, String) {
        let vec_arg: Vec<&str> = args.split(',').collect();
        (
            vec_arg[0].parse().expect("Cannot parse first arg"),
            vec_arg[1].parse().expect("Cannot parse second arg"),
            vec_arg[2].trim().to_owned(),
        )
    }

    fn process_args_1i(&self, args: &str, _instr: &str) -> (usize, String) {
        let vec_arg: Vec<&str> = args.split(',').collect();
        (
            vec_arg[0].parse().expect("Cannot parse first arg"),
            vec_arg[1].trim().to_owned(),
        )
    }

    fn process_args_2(&self, args: &str, _instr: &str) -> Vec<usize> {
        let vec_arg = args
            .split(',')
            .map(|x| x.parse::<usize>().expect("Cannot parse int"))
            .collect();
        vec_arg
    }

    fn process_string_args(&self, arg: &str) -> Option<i16> {
        if let Some(result) = arg.strip_prefix("0x") {
            match i32::from_str_radix(result, 16) {
                Ok(result) => Some(result as i16),
                _ => None,
            }
        } else if let Some(result) = arg.strip_prefix("0b") {
            match i32::from_str_radix(result, 2) {
                Ok(result) => Some(result as i16),
                _ => None,
            }
        } else if let Ok(result) = arg.parse::<i32>() {
            Some(result as i16)
        } else {
            None
        }
    }

    fn halt(&self, _args: &str) -> Option<bool> {
        None
    }

    fn nop(&self, _args: &str) -> Option<bool> {
        Some(true)
    }

    fn reset(&mut self, _args: &str) -> Option<bool> {
        self.jalr(vec![0, 0])
    }

    fn add(&mut self, args: Vec<usize>) -> Option<bool> {
        self.registers[args[0]] = self.registers[args[1]].wrapping_add(self.registers[args[2]]);
        Some(true)
    }

    fn addi(&mut self, args: (usize, usize, String)) -> Option<bool> {
        let imm = self
            .process_string_args(&args.2)
            .expect("Error processing label/imm");
        if imm > 63 || imm < -64 {
            println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm).unwrap();
        }
        self.registers[args.0] = self.registers[args.1].wrapping_add(imm);
        Some(true)
    }

    fn nand(&mut self, args: Vec<usize>) -> Option<bool> {
        self.registers[args[0]] = !(self.registers[args[1]] & self.registers[args[2]]);
        Some(true)
    }

    fn movi(&mut self, args: (usize, String)) -> Option<bool> {
        self.registers[args.0] = self
            .process_string_args(&args.1)
            .expect("Error processing label/imm");
        Some(true)
    }

    fn lui(&mut self, args: (usize, String)) -> Option<bool> {
        let imm = self
            .process_string_args(&args.1)
            .expect("Error processing label/imm");
        if imm > 1023 || imm < 0 {
            println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm).unwrap();
        }
        self.registers[args.0] = imm.wrapping_shl(5);
        Some(true)
    }

    fn lw(&mut self, args: (usize, usize, String)) -> Option<bool> {
        let imm = self
            .process_string_args(&args.2)
            .expect("Error processing label/imm");
        if imm > 63 || imm < -64 {
            println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm).unwrap();
        }
        self.registers[args.0] = self.ram[self.registers[args.0] as usize + imm as usize];
        Some(true)
    }

    fn sw(&mut self, args: (usize, usize, String)) -> Option<bool> {
        let imm = self
            .process_string_args(&args.2)
            .expect("Error processing label/imm");
        if imm > 63 || imm < -64 {
            println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm).unwrap();
        }
        self.ram[self.registers[args.0] as usize + imm as usize] = self.registers[args.0];
        Some(true)
    }

    fn beq(&mut self, args: (usize, usize, String)) -> Option<bool> {
        if self.registers[args.1] == self.registers[args.0] {
            let lab;
            match self.labels.get(&args.2) {
                Some(res) => lab = *res as i32 - 1,
                None => match self.process_string_args(&args.2) {
                    Some(res) => lab = res.into(),
                    _ => {
                        println!("Impossible to parse jump");
                        writeln!(self.buffer,"Impossible to parse jump").unwrap();
                        return None;
                    }
                },
            };
            if lab - (self.pc as i32) < -64 || lab - self.pc as i32 > 63 {
                let jump = lab - self.pc as i32;
                println!("WARNING, Jump too long: \"{}\" of size {}", &args.2, jump);
                writeln!(self.buffer,"WARNING, Jump too long: \"{}\" of size {}", &args.2, jump).unwrap();
            }
            self.pc = lab as usize;
            println!("Jumping to: {}: {}", self.pc, &args.2);
            writeln!(self.buffer,"Jumping to: {}: {}", self.pc, &args.2).unwrap();
        }
        Some(true)
    }

    fn jalr(&mut self, args: Vec<usize>) -> Option<bool> {
        self.registers[args[0]] = self.pc as i16 + 1;
        self.pc = self.registers[args[1]] as usize - 1;
        Some(true)
    }
}

fn load_rom(content: String) -> (Vec<String>, HashMap<String, usize>) {
    let re_cmts = Regex::new(r"(?m)//.*?$").unwrap();
    let code_without_comments = re_cmts.replace_all(&content, "\n");
    let re_labop = Regex::new(r"^(\S*):(.*)").unwrap();
    let mut instr: Vec<String> = Vec::new();
    let mut labels = HashMap::new();
    let mut instr_counter = 0;
    for line in code_without_comments.lines() {
        if line.ends_with(':') {
            labels.insert(line.replace(":", ""), instr_counter);
        // println!("lab only: {}, i {}", line, instr_counter)
        } else if re_labop.is_match(line.trim()) {
            let cap = re_labop.captures(line.trim()).unwrap();
            instr.push(cap[2].trim().to_owned());
            labels.insert(cap[1].trim().to_owned(), instr_counter);
            instr_counter += 1;
        // println!("lab: {}, i {}", line, instr_counter)
        } else if line.trim() == "" {
            continue;
        } else {
            instr.push(line.trim().to_string());
            instr_counter += 1;
            // println!("instr: {}", line)
        }
    }
    (instr, labels)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("Lauching: {}", filename);

    let content = fs::read_to_string(filename).expect("Error reading file");

    let mut proc = Risc16::build(Archtype::IS0);

    let (rom, labels) = load_rom(content);
    println!("{:?}", rom);
    println!("{:?}", labels);
    proc.execute(&rom, labels);

    proc.display_state(true);
}

pub fn main_from_str(code: &str) -> String{
    let mut proc = Risc16::build(Archtype::IS0);

    let (rom, labels) = load_rom(code.to_string());
    println!("{:?}", rom);
    println!("{:?}", labels);
    proc.execute(&rom, labels);

    proc.display_state(true);
    proc.buffer
}
