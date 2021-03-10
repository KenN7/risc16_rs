use lazy_static::lazy_static;
use pyo3::exceptions::PyBaseException;
use pyo3::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs;

#[derive(Debug)]
enum CustomError {
    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    Regex(regex::Error),
    Format(std::fmt::Error),
    Instr(String),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::Io(ref err) => err.fmt(f),
            CustomError::ParseInt(ref err) => err.fmt(f),
            CustomError::ParseFloat(ref err) => err.fmt(f),
            CustomError::Regex(ref err) => err.fmt(f),
            CustomError::Format(ref err) => err.fmt(f),
            CustomError::Instr(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<CustomError> for PyErr {
    fn from(err: CustomError) -> PyErr {
        PyBaseException::new_err(err.to_string())
    }
}

impl From<&str> for CustomError {
    fn from(err: &str) -> CustomError {
        CustomError::Instr(err.to_string())
    }
}

impl From<String> for CustomError {
    fn from(err: String) -> CustomError {
        CustomError::Instr(err)
    }
}

impl From<std::fmt::Error> for CustomError {
    fn from(err: std::fmt::Error) -> CustomError {
        CustomError::Format(err)
    }
}

impl From<std::io::Error> for CustomError {
    fn from(err: std::io::Error) -> CustomError {
        CustomError::Io(err)
    }
}

impl From<regex::Error> for CustomError {
    fn from(err: regex::Error) -> CustomError {
        CustomError::Regex(err)
    }
}

impl From<std::num::ParseIntError> for CustomError {
    fn from(err: std::num::ParseIntError) -> CustomError {
        CustomError::ParseInt(err)
    }
}

impl From<std::num::ParseFloatError> for CustomError {
    fn from(err: std::num::ParseFloatError) -> CustomError {
        CustomError::ParseFloat(err)
    }
}

type RiscResult<T> = std::result::Result<T, CustomError>;

#[pyclass]
struct Risc16 {
    #[pyo3(get)]
    registers: [i16; 8],
    #[pyo3(get)]
    pc: usize,
    ram: [i16; 256],
    #[pyo3(get)]
    instr_count: u32,
    #[pyo3(get)]
    max_instr: u32,
    #[pyo3(get)]
    labels: HashMap<String, usize>,
    arch: Archtype,
    #[pyo3(get)]
    buffer: String,
}

enum Archtype {
    IS0,
    IS1,
    IS2,
}

impl Risc16 {
    fn new(arch: Archtype, max_instr: u32) -> Risc16 {
        Risc16 {
            registers: [0; 8],
            pc: 0,
            ram: [0; 256],
            instr_count: 0,
            max_instr,
            labels: HashMap::new(),
            arch,
            buffer: String::new(),
        }
    }

    fn execute(
        &mut self,
        rom: &[(String, Args)],
        labels: &HashMap<String, usize>,
    ) -> RiscResult<bool> {
        self.labels = labels.to_owned();
        for instr in 0..=self.max_instr {
            let halt = self.execute_instr(
                rom.get(self.pc)
                    .ok_or("Reaching end of ROM, missing HALT")?,
            )?;
            self.registers[0] = 0;
            if !halt {
                break;
            } else if instr == self.max_instr {
                return Err(CustomError::Instr(
                    "Reaching max instruction count, missing HALT or infinite loop ?".into(),
                ));
            }
            self.instr_count += 1;
            self.pc += 1;
        }
        Ok(true)
    }

    fn execute_instr(&mut self, full_instr: &(String, Args)) -> RiscResult<bool> {
        // self.display_state(false);
        let (instr, args) = full_instr;
        // FIXME TODO exec trace:
        // println!("{} {}", instr, args);
        // writeln!(self.buffer, "{} {}", instr, args)?;

        match instr.as_str() {
            "nop" => self.nop(args).ok_or("nop error".into()),
            "halt" => self.halt(args).ok_or("halt error".into()),
            "reset" => self.reset(args),
            "add" => self.add(args),
            "addi" => self.addi(args),
            "nand" => self.nand(args),
            "movi" => self.movi(args),
            "lui" => self.lui(args),
            "lw" => self.lw(args),
            "sw" => self.sw(args),
            "beq" => self.beq(args),
            "jalr" => self.jalr(args),
            _ => {
                // println!("Error: Instr not know: {}", instr);
                Err("Error: Instr not know".into())
            }
        }
    }

    fn reset_state(&mut self) {
        self.registers = [0; 8];
        self.pc = 0;
        self.ram = [0; 256];
        self.instr_count = 0;
        self.labels = HashMap::new();
        self.buffer = String::new();
    }

    fn display_state(&mut self, full: bool) {
        print!("PC: {}, Instr. count: {}", self.pc, self.instr_count);
        write!(
            self.buffer,
            "PC: {}, Instr. count: {}",
            self.pc, self.instr_count
        )
        .unwrap();
        println!(", regs: {:x?}", self.registers);
        writeln!(self.buffer, ", regs: {:x?}", self.registers).unwrap();
        if full {
            println!("ram: {:?}", self.ram);
        }
    }

    fn print_state(&mut self, full: bool) -> RiscResult<String> {
        let mut state: String = String::from("");
        write!(state, "PC: {}, Instr. count: {}", self.pc, self.instr_count)?;
        writeln!(state, ", regs: {:x?}", self.registers)?;
        if full {
            writeln!(state, "ram: {:?}", self.ram)?;
        }
        Ok(state)
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

    fn halt(&self, _args: &Args) -> Option<bool> {
        Some(false)
    }

    fn nop(&self, _args: &Args) -> Option<bool> {
        Some(true)
    }

    fn reset(&mut self, _args: &Args) -> RiscResult<bool> {
        //&str
        self.jalr(&Args::A23(vec![0, 0]))?;
        Ok(true)
    }

    fn add(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A23(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //Vec<usize>
        let val1 = *self.registers.get(args[1]).ok_or("")?;
        let val2 = *self.registers.get(args[2]).ok_or("")?;
        let reg = self
            .registers
            .get_mut(args[0])
            .ok_or("Index of register out of bounds.")?;
        *reg = val1.wrapping_add(val2);
        Ok(true)
    }

    fn addi(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A2i(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //(usize, usize, String)
        let imm = self
            .process_string_args(&args.2)
            .ok_or("Error processing label/imm")?;
        if imm > 63 || imm < -64 {
            // println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm).unwrap();
        }
        let val = *self.registers.get(args.1).ok_or("")?;
        let reg = self
            .registers
            .get_mut(args.0)
            .ok_or("Index of register out of bounds.")?;
        *reg = val.wrapping_add(imm);
        Ok(true)
    }

    fn nand(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A23(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //Vec<usize>
        let val1 = *self.registers.get(args[1]).ok_or("")?;
        let val2 = *self.registers.get(args[2]).ok_or("")?;
        let reg = self
            .registers
            .get_mut(args[0])
            .ok_or("Index of register out of bounds.")?;
        *reg = !(val1 & val2);
        Ok(true)
    }

    fn movi(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A1i(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //(usize, String)
        let val = self
            .process_string_args(&args.1)
            .ok_or("Error processing label/imm")?;
        let reg = self
            .registers
            .get_mut(args.0)
            .ok_or("Index of register out of bounds.")?;
        *reg = val;
        Ok(true)
    }

    fn lui(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A1i(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //(usize, String)
        let imm = self
            .process_string_args(&args.1)
            .ok_or("Error processing label/imm")?;
        if imm > 1023 || imm < 0 {
            // println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm)?;
        }
        let reg = self
            .registers
            .get_mut(args.0)
            .ok_or("Index of register out of bounds.")?;
        *reg = imm.wrapping_shl(5);
        Ok(true)
    }

    fn lw(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A2i(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //(usize, usize, String)
        let imm = self
            .process_string_args(&args.2)
            .ok_or("Error processing label/imm")?;
        if imm > 63 || imm < -64 {
            // println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm)?;
        }
        let address = *self.registers.get(args.1).ok_or("")? + imm;
        let reg = self
            .registers
            .get_mut(args.0)
            .ok_or("Index of register out of bounds.")?;
        let val = *self
            .ram
            .get(address as usize)
            .ok_or("Index of memory out of bounds.")?;
        *reg = val;
        Ok(true)
    }

    fn sw(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A2i(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //(usize, usize, String)
        let imm = self
            .process_string_args(&args.2)
            .ok_or("Error processing label/imm")?;
        if imm > 63 || imm < -64 {
            // println!("/!\\ Immediate Too BIG : {}", imm);
            writeln!(self.buffer, "/!\\ Immediate Too BIG : {}", imm)?;
        }
        let address = *self.registers.get(args.1).ok_or("")? + imm;
        let ram = self
            .ram
            .get_mut(address as usize)
            .ok_or("Index of memory out of bounds.")?;
        *ram = *self.registers.get(args.0).ok_or("")?;
        Ok(true)
    }

    fn beq(&mut self, args: &Args) -> RiscResult<bool> {
        let args = match args {
            Args::A2i(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        //(usize, usize, String)
        if self.registers.get(args.1).ok_or("")? == self.registers.get(args.0).ok_or("")? {
            let lab;
            match self.labels.get(&args.2) {
                Some(res) => lab = *res as i32 - 1,
                None => match self.process_string_args(&args.2) {
                    Some(res) => lab = res.into(),
                    _ => {
                        // println!("Impossible to parse jump");
                        writeln!(self.buffer, "Impossible to parse jump")?;
                        return Err("Impossible to parse jump".into());
                    }
                },
            }
            if lab - (self.pc as i32) < -64 || lab - self.pc as i32 > 63 {
                let jump = lab - self.pc as i32;
                // println!("WARNING, Jump too long: \"{}\" of size {}", &args.2, jump);
                writeln!(
                    self.buffer,
                    "WARNING, Jump too long: \"{}\" of size {}",
                    &args.2, jump
                )?;
            }
            self.pc = lab as usize;
            // println!("Jumping to: {}: {}", self.pc, &args.2);
            //FIXME TODO exec trace
            // writeln!(self.buffer, "Jumping to: {}: {}", self.pc, &args.2)?;
        }
        Ok(true)
    }

    fn jalr(&mut self, args: &Args) -> RiscResult<bool> {
        //vec
        let args = match args {
            Args::A23(a) => a,
            _ => return Err("Bad argument types".into()),
        };
        let val = *self
            .registers
            .get(args[1])
            .ok_or("Index of register out of bounds.")?;
        let reg = self
            .registers
            .get_mut(args[0])
            .ok_or("Index of register out of bounds.")?;
        *reg = self.pc as i16 + 1;
        self.pc = val as usize - 1;
        Ok(true)
    }
}

#[derive(Debug)]
enum Args {
    A23(Vec<usize>),
    A2i((usize, usize, String)),
    A1i((usize, String)),
    None(bool),
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Args::A23(a) => {
                write!(
                    f,
                    "{}",
                    a.iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            Args::A1i(a) => write!(f, "{},{}", a.0, a.1),
            Args::A2i(a) => write!(f, "{},{},{}", a.0, a.1, a.2),
            Args::None(_a) => write!(f, ""),
        }
    }
}

fn process_args_vec(args: &str, len: usize) -> RiscResult<Args> {
    let vec_arg = args
        .split(',')
        .map(|x| x.trim().parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CustomError::from(e))?;
    if vec_arg.len() != len {
        return Err("Wrong number of arguments".into());
    }
    Ok(Args::A23(vec_arg))
}

fn process_args_2i(args: &str) -> RiscResult<Args> {
    let vec_arg: Vec<&str> = args.split(',').collect();
    Ok(Args::A2i((
        vec_arg[0].trim().parse()?,
        vec_arg[1].trim().parse()?,
        vec_arg[2].trim().to_owned(),
    )))
}

fn process_args_1i(args: &str) -> RiscResult<Args> {
    let vec_arg: Vec<&str> = args.split(',').collect();
    Ok(Args::A1i((
        vec_arg[0].trim().parse()?,
        vec_arg[1].trim().to_owned(),
    )))
}

fn process_line(line: &str) -> RiscResult<(String, Args)> {
    lazy_static! {
        static ref RE_SPC: Regex = Regex::new(r"\s+").unwrap();
    }
    let vec_instr: Vec<&str> = RE_SPC.splitn(line.trim(), 2).collect();
    let instr = vec_instr
        .get(0)
        .ok_or(format!("get error: {:?}", vec_instr))?;
    let args = vec_instr
        .get(1)
        // .ok_or(format!("get error: {:?}", vec_instr))?;
        .unwrap_or(&"");
    let processed_args = match *instr {
        "nop" => Args::None(true),
        "halt" => Args::None(true),
        "reset" => Args::None(true),
        "add" => process_args_vec(args, 3)?,
        "addi" => process_args_2i(args)?,
        "nand" => process_args_vec(args, 3)?,
        "movi" => process_args_1i(args)?,
        "lui" => process_args_1i(args)?,
        "lw" => process_args_2i(args)?,
        "sw" => process_args_2i(args)?,
        "beq" => process_args_2i(args)?,
        "jalr" => process_args_vec(args, 2)?,
        _ => {
            // println!("Error: Instr not know: {}", instr);
            // writeln!(self.buffer,"Error: Instr not know: {}", instr).unwrap();
            return Err("Error: Instruction unknow".into());
            // Args::None(false)
        }
    };
    Ok((instr.to_string(), processed_args))
}

fn load_rom(content: String) -> RiscResult<(Vec<(String, Args)>, HashMap<String, usize>)> {
    let re_cmts = Regex::new(r"(?m)//.*?$")?;
    let code_without_comments = re_cmts.replace_all(&content, "\n");
    let re_labop = Regex::new(r"^(\S*):(.*)")?;
    let mut instr: Vec<(String, Args)> = Vec::new();
    let mut labels = HashMap::new();
    let mut instr_counter = 0;
    for line in code_without_comments.lines() {
        if line.ends_with(':') {
            labels.insert(line.replace(":", ""), instr_counter);
        // println!("lab only: {}, i {}", line, instr_counter)
        } else if re_labop.is_match(line.trim()) {
            let cap = re_labop.captures(line.trim()).ok_or("Regex Problem")?;
            labels.insert(cap[1].trim().to_owned(), instr_counter);
            let l = match process_line(cap[2].trim()) {
                Ok(l) => l,
                Err(e) => return Err(format!("{}: {}", e, cap[2].trim()).into()),
            };
            instr.push(l);
            instr_counter += 1;
        } else if line.trim() == "" {
            continue;
        } else {
            let l = match process_line(line.trim()) {
                Ok(l) => l,
                Err(e) => return Err(format!("{}: {}", e, line.trim()).into()),
            };
            instr.push(l);
            instr_counter += 1;
        }
    }
    Ok((instr, labels))
}

fn format_code(instr: Vec<(String, Args)>, labels: HashMap<String, usize>) -> Vec<String> {
    let mut code_vec = instr
        .iter()
        .map(|(s, args)| format!("{} {}", s, args))
        .collect::<Vec<_>>();

    for l in labels.iter() {
        let s = code_vec.get_mut(*l.1).unwrap();
        *s = format!("{}: {}", l.0, s);
    }
    code_vec
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("Lauching: {}", filename);

    let content = fs::read_to_string(filename).expect("Error reading file");

    let mut proc = Risc16::new(Archtype::IS0, 100000);

    let (rom, labels) = load_rom(content).unwrap();
    println!("{:?}", rom);
    println!("{:?}", labels);
    match proc.execute(&rom, &labels) {
        Ok(_res) => println!("Success !"),
        Err(e) => {
            writeln!(proc.buffer, "Error! {}", e).unwrap();
            println!("Error! {}", e)
        }
    }
    proc.display_state(true);
}

pub fn main_from_str(code: &str) -> String {
    let mut proc = Risc16::new(Archtype::IS0, 100000);

    let (rom, labels) = load_rom(code.to_string()).unwrap();
    println!("{:?}", rom);
    println!("{:?}", labels);
    match proc.execute(&rom, &labels) {
        Ok(_res) => println!("Success !"),
        Err(e) => {
            writeln!(proc.buffer, "Error! {}", e).unwrap();
            // println!("Error! {}", e)
        }
    }
    proc.display_state(true);
    proc.buffer
}

#[pymodule]
fn librisc16_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "run_from_str_py")]
    fn run_from_str_py(
        _py: Python,
        max_instr: u32,
        trace: bool,
        code: &str,
    ) -> PyResult<(String, String)> {
        let mut proc = Risc16::new(Archtype::IS0, max_instr);
        let (rom, labels) = match load_rom(code.to_string()) {
            Ok((rom, labels)) => (rom, labels),
            Err(e) => return Err(PyErr::from(e)),
        };
        match proc.execute(&rom, &labels) {
            Ok(_res) => (), //println!("Success !"),
            Err(e) => {
                writeln!(proc.buffer, "Error! {}", e).unwrap();
                // println!("Error! {}", e)
            }
        }
        Ok((proc.buffer.to_string(), proc.print_state(false)?))
    }

    #[pyfn(m, "test_batch_py")]
    fn test_batch_py(
        _py: Python,
        max_instr: u32,
        trace: bool,
        code: &str,
        tests: Vec<Vec<(i32, i32)>>,
    ) -> PyResult<Vec<[i16; 8]>> {
        let mut proc = Risc16::new(Archtype::IS0, max_instr);
        let (rom, labels) = match load_rom(code.to_string()) {
            Ok((rom, labels)) => (rom, labels),
            Err(e) => return Err(PyErr::from(e)),
        };

        let mut outputs = Vec::new();
        for test in tests {
            proc.reset_state();
            for input in test {
                proc.registers[input.0 as usize] = input.1 as i16
            }
            match proc.execute(&rom, &labels) {
                Ok(_res) => (), //println!("Success !"),
                Err(e) => {
                    writeln!(proc.buffer, "Error! {}", e).unwrap();
                }
            }
            outputs.push(proc.registers)
        }
        Ok(outputs)
    }

    #[pyfn(m, "test_batch_par_py")]
    fn test_batch_par_py(
        py: Python,
        max_instr: u32,
        trace: bool,
        code: &str,
        tests: Vec<Vec<(i32, i32)>>,
        // ) -> PyResult<Vec<[i16; 8]>> {
    ) -> PyResult<Vec<Risc16>> {
        let (rom, labels) = match load_rom(code.to_string()) {
            Ok((rom, labels)) => (rom, labels),
            Err(e) => return Err(PyErr::from(e)),
        };

        py.allow_threads(|| {
            let outputs = tests
                .par_iter()
                .map(|test| {
                    let mut proc = Risc16::new(Archtype::IS0, max_instr);
                    for input in test {
                        proc.registers[input.0 as usize] = input.1 as i16
                    }
                    match proc.execute(&rom, &labels) {
                        Ok(_res) => (), //println!("Success !"),
                        Err(e) => {
                            writeln!(proc.buffer, "Error! {}", e).unwrap();
                        }
                    }
                    // return proc.registers;
                    return proc;
                })
                .collect::<Vec<_>>();
            Ok(outputs)
        })
    }

    #[pyfn(m, "load_rom_py")]
    fn load_rom_py<'a>(_py: Python, code: &str) -> PyResult<String> {
        match load_rom(code.to_string()) {
            Ok((rom, labels)) => {
                let verified_code = format_code(rom, labels);
                Ok(verified_code.join("\n"))
            }
            Err(e) => Err(PyErr::from(e)),
        }
    }

    Ok(())
}
