use regex::Regex;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::env;
use std::fs;

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
}

impl Risc16 {
    fn build(arch: Archtype) -> Risc16 {
        Risc16 {
            registers: [0; 8],
            pc: 0,
            ram: [0; 256],
            instr_count: 0,
            max_instr: 1000,
            labels: HashMap::new(),
            arch,
        }
    }
    fn execute(&mut self, rom: &Vec<String>, labels: HashMap<String, usize>) {
        self.labels = labels;
        for _instr in 0..self.max_instr {
            if !self.execute_instr(rom.get(self.pc).unwrap()) {
                break;
            }

            self.instr_count += 1;
            self.pc += 1;
            self.registers[0] = 0;
        }
    }

    fn execute_instr(&mut self, full_instr: &str) -> bool {
        lazy_static! {
            static ref RE_SPC: Regex = Regex::new(r"\s+").unwrap();
        }    
        let vec_instr: Vec<&str> = RE_SPC.splitn(full_instr, 2).collect();
        let instr = vec_instr.get(0).unwrap();
        let args = vec_instr.get(1).unwrap_or(&"");
        self.display_state(false);
        println!("exec: {}({})", instr, args);

        match instr.as_ref() {
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
                return true;
            }
        }
    }

    fn display_state(&self, full: bool) {
        print!("PC: {}, Instr. count: {}", self.pc, self.instr_count);
        println!(", regs: {:?}", self.registers);
        if full {
            println!("ram: {:?}", self.ram);
        }
    }

    fn process_args_3(&self, args: &str, _instr: &str) -> Vec<usize> {
        let vec_arg = args
            .split(',')
            .map(|x| x.parse::<usize>().unwrap())
            .collect();
        vec_arg
    }

    fn process_args_2i(&self, args: &str, _instr: &str) -> (usize, usize, String) {
        let vec_arg: Vec<&str> = args.split(',').collect();
        (
            vec_arg[0].parse().unwrap(),
            vec_arg[1].parse().unwrap(),
            vec_arg[2].trim().to_owned(),
        )
    }

    fn process_args_1i(&self, args: &str, _instr: &str) -> (usize, String) {
        let vec_arg: Vec<&str> = args.split(',').collect();
        (vec_arg[0].parse().unwrap(), vec_arg[1].trim().to_owned())
    }

    fn process_args_2(&self, args: &str, _instr: &str) -> Vec<usize> {
        let vec_arg = args
            .split(',')
            .map(|x| x.parse::<usize>().unwrap())
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

    fn halt(&self, _args: &str) -> bool {
        false
    }

    fn nop(&self, _args: &str) -> bool {
        true
    }

    fn reset(&mut self, _args: &str) -> bool {
        self.jalr(vec![0, 0]);
        true
    }

    fn add(&mut self, args: Vec<usize>) -> bool {
        self.registers[args[0]] = self.registers[args[1]].wrapping_add(self.registers[args[2]]);
        true
    }

    fn addi(&mut self, args: (usize, usize, String)) -> bool {
        self.registers[args.0] =
            self.registers[args.1].wrapping_add(self.process_string_args(&args.2).unwrap());
        true
    }

    fn nand(&mut self, args: Vec<usize>) -> bool {
        self.registers[args[0]] = !(self.registers[args[1]] & self.registers[args[2]]);
        true
    }

    fn movi(&mut self, args: (usize, String)) -> bool {
        self.registers[args.0] = self.process_string_args(&args.1).unwrap();
        true
    }

    fn lui(&mut self, args: (usize, String)) -> bool {
        self.registers[args.0] = self.process_string_args(&args.1).unwrap().wrapping_shl(5);
        true
    }

    fn lw(&mut self, args: (usize, usize, String)) -> bool {
        self.registers[args.0] = self.ram
            [self.registers[args.0] as usize + self.process_string_args(&args.2).unwrap() as usize];
        true
    }

    fn sw(&mut self, args: (usize, usize, String)) -> bool {
        self.ram[self.registers[args.0] as usize
            + self.process_string_args(&args.2).unwrap() as usize] = self.registers[args.0];
        true
    }

    fn beq(&mut self, args: (usize, usize, String)) -> bool {
        if self.registers[args.1] == self.registers[args.0] {
            let lab = self.labels.get(&args.2).unwrap() - 1;
            // self.pc = self.registers[lab.to_owned()] as usize;
            self.pc = lab.to_owned() as usize;
            println!("Jumping to: {}: {}", self.pc, &args.2);
        }
        true
    }

    fn jalr(&mut self, args: Vec<usize>) -> bool {
        self.registers[args[0]] = self.pc as i16 + 1;
        self.pc = self.registers[args[1]] as usize - 1;
        true
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
        if line.ends_with(":") {
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
