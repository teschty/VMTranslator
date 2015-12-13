#[derive(Debug)]
pub enum VMCommand {
    Nothing,

    // Arithmetic/Boolean commands
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,

    // Memory access commands
    Pop(VMMemorySegment, i32),
    Push(VMMemorySegment, i32),

    // Program flow commands
    Label(String),
    GoTo(String),
    IfGoTo(String),

    // Function calling commands
    Function(String, i32),
    Call(String, i32),
    Return,
}

#[derive(Debug)]
pub enum VMMemorySegment {
    Static,
    This,
    Local,
    Argument,
    That,
    Constant,
    Pointer,
    Temp,
}

pub struct Parser {
    pub input: Vec<String>,
    pub index: usize,
}

impl Parser {
    pub fn advance(&mut self) -> Result<VMCommand, &'static str> {
        // we're supposed to iter over the lines
        // so reaching the end of the file is an error
        if self.eof() {
            return Err("end of file reached");
        }

        // remove comments
        let line = match self.input[self.index].find("//") {
            Some(n) => &self.input[self.index][..n],
            None => &self.input[self.index][..],
        };

        let parts: Vec<&str> = line.split_whitespace().collect();
        self.index += 1;

        // if this is an empty line
        if parts.len() < 1 {
            return Ok(VMCommand::Nothing);
        }

        match parts[0] {
            "add" => Ok(VMCommand::Add),
            "sub" => Ok(VMCommand::Sub),
            "neg" => Ok(VMCommand::Neg),
            "eq" => Ok(VMCommand::Eq),
            "gt" => Ok(VMCommand::Gt),
            "lt" => Ok(VMCommand::Lt),
            "and" => Ok(VMCommand::And),
            "or" => Ok(VMCommand::Or),
            "not" => Ok(VMCommand::Not),
            "return" => Ok(VMCommand::Return),
            "if-goto" => Ok(VMCommand::IfGoTo(parts[1].to_string())),
            "label" => Ok(VMCommand::Label(parts[1].to_string())),
            "goto" => Ok(VMCommand::GoTo(parts[1].to_string())),
            "push" | "pop" => {
                let segment = match parts[1] {
                    "static" => VMMemorySegment::Static,
                    "this" => VMMemorySegment::This,
                    "local" => VMMemorySegment::Local,
                    "argument" => VMMemorySegment::Argument,
                    "that" => VMMemorySegment::That,
                    "constant" => VMMemorySegment::Constant,
                    "pointer" => VMMemorySegment::Pointer,
                    "temp" => VMMemorySegment::Temp,
                    _ => return Err("invalid memory segment"),
                };

                let arg = match parts[2].parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => return Err("invalid numeric argument"),
                };

                match parts[0] {
                    "push" => Ok(VMCommand::Push(segment, arg)),
                    "pop" => Ok(VMCommand::Pop(segment, arg)),
                    _ => Err("this should never happen"),
                }
            }
            "function" | "call" => {
                let arg = match parts[2].parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => return Err("invalid numeric argument"),
                };

                match parts[0] {
                    "function" => Ok(VMCommand::Function(parts[1].to_string(), arg)),
                    "call" => Ok(VMCommand::Call(parts[1].to_string(), arg)),
                    _ => Err("this should never happen"),
                }
            }
            _ => Ok(VMCommand::Nothing),
        }
    }

    pub fn new(input: Vec<String>) -> Parser {
        Parser {
            input: input,
            index: 0,
        }
    }

    pub fn eof(&self) -> bool {
        self.index >= self.input.len()
    }
}
