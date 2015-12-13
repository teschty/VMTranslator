use std::io;
use std::fs::File;
use std::io::prelude::*;
use vm_parser::{VMCommand, VMMemorySegment};

pub struct CodeWriter {
    output: File,
    jump_flag: i32,
    label_count: i32,
    pub file_name: String
}

impl CodeWriter {
    pub fn new(file_name: &str) -> io::Result<CodeWriter> {
        let file = try!(File::create(file_name));

        Ok(CodeWriter {
            output: file,
            jump_flag: 0,
            label_count: 0,
            file_name: "".to_string()
        })
    }

    pub fn write_init(&mut self) -> io::Result<bool> {
        let bytes = format!("@256\nD=A\n@SP\nM=D\n{}", self.call_template("Sys.init", 0)).into_bytes();

        match self.output.write_all(&bytes[..]) {
            Ok(_) => Ok(true),
            Err(e) => Err(e)
        }
    }

    pub fn write_command(&mut self, cmd: VMCommand) -> io::Result<bool> {
        let out = match cmd {
            VMCommand::Add => self.arithmetic_template() + "M=M+D\n",
            VMCommand::Sub => self.arithmetic_template() + "M=M-D\n",
            VMCommand::And => self.arithmetic_template() + "M=M&D\n",
            VMCommand::Or => self.arithmetic_template() + "M=M|D\n",
            VMCommand::Gt => self.arithmetic_template_jmp("JLE"),
            VMCommand::Lt => self.arithmetic_template_jmp("JGE"),
            VMCommand::Eq => self.arithmetic_template_jmp("JNE"),
            VMCommand::Not => "@SP\nA=M-1\nM=!M\n".to_string(),
            VMCommand::Neg => "D=0\n@SP\nA=M-1\nM=D-M\n".to_string(),

            // push/pop
            VMCommand::Push(seg, i) => {
                match seg {
                    VMMemorySegment::Constant =>
                        format!("@{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", i),
                    VMMemorySegment::Local => self.push_template("LCL", i, false),
                    VMMemorySegment::Argument => self.push_template("ARG", i, false),
                    VMMemorySegment::This => self.push_template("THIS", i, false),
                    VMMemorySegment::That => self.push_template("THAT", i, false),
                    VMMemorySegment::Temp => self.push_template("R5", i + 5, false),
                    VMMemorySegment::Pointer => {
                        if i == 0 {
                            self.push_template("THIS", i, true)
                        } else {
                            self.push_template("THAT", i, true)
                        }
                    }
                    VMMemorySegment::Static =>
                        format!("@{}{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", self.file_name, i)
                }
            }
            VMCommand::Pop(seg, i) => {
                match seg {
                    VMMemorySegment::Local => self.pop_template("LCL", i, false),
                    VMMemorySegment::Argument => self.pop_template("ARG", i, false),
                    VMMemorySegment::This => self.pop_template("THIS", i, false),
                    VMMemorySegment::That => self.pop_template("THAT", i, false),
                    VMMemorySegment::Temp => self.pop_template("R5", i + 5, false),
                    VMMemorySegment::Pointer => {
                        if i == 0 {
                            self.pop_template("THIS", i, true)
                        } else {
                            self.pop_template("THAT", i, true)
                        }
                    }
                    VMMemorySegment::Static =>
                        format!("@{}{}\nD=A\n@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n", self.file_name, i),
                    _ => "".to_string(),
                }
            }

            //function calls and whatnot
            VMCommand::Function(func_name, n) => {
                self.write_func(&func_name[..], n)
            }
            VMCommand::Call(func_name, n) => {
                self.call_template(&func_name[..], n)
            }
            VMCommand::IfGoTo(label) => {
                self.if_template(&label[..])
            }
            VMCommand::Return => {
                self.return_template()
            }
            VMCommand::GoTo(label) => {
                self.goto_template(&label[..])
            }
            VMCommand::Label(label) => {
                self.label_template(&label[..])
            }
            _ => "".to_string(),
        };

        let bytes = out.into_bytes();
        match self.output.write_all(&bytes[..]) {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    fn label_template(&self, label: &str) -> String {
        format!("({})\n", label)
    }

    fn goto_template(&self, label: &str) -> String {
        format!("@{}\n0;JMP\n", label)
    }

    fn if_template(&self, label: &str) -> String {
        format!("{}@{}\nD;JNE\n", self.arithmetic_template(), label)
    }

    fn call_template(&mut self, func_name: &str, num_args: i32) -> String {
        self.label_count += 1;
        format!(
            "@RETURN_LABEL{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n\
            {}{}{}{}@SP\nD=M\n@5\nD=D-A\n@{}\nD=D-A\n@ARG\nM=D\n\
            @SP\nD=M\n@LCL\nM=D\n@{}\n0;JMP\n(RETURN_LABEL{})\n",
            self.label_count - 1, self.push_template("LCL", 0, true),
            self.push_template("ARG", 0, true), self.push_template("THIS", 0, true),
            self.push_template("THAT", 0, true), num_args, func_name, self.label_count - 1)
    }

    fn write_func(&mut self, func_name: &str, num_locals: i32) -> String {
        let bytes = format!("({})\n", func_name).into_bytes();

        match self.output.write_all(&bytes[..]) {
            Ok(_) => {},
            Err(e) => panic!("panic in write_func! {}", e)
        }

        for _ in 0..num_locals {
            match self.write_command(VMCommand::Push(VMMemorySegment::Constant, 0)) {
                Ok(_) => {},
                Err(e) => panic!("error creating locals! {}", e),
            }
        }

        "".to_string()
    }

    fn arithmetic_template(&self) -> String {
        "@SP\nAM=M-1\nD=M\nA=A-1\n".to_string()
    }

    fn save_template(&self, pos: &str) -> String {
        format!("@R11\nD=M-1\nAM=D\nD=M\n@{}\nM=D\n", pos)
    }

    fn return_template(&mut self) -> String {
        format!("@LCL\nD=M\n@R11\nM=D\n@5\nA=D-A\nD=M\n@R12\n\
            M=D\n{}@ARG\nD=M\n@SP\nM=D+1\n{}{}{}{}@R12\nA=M\n0;JMP\n",
            self.pop_template("ARG", 0, false),
            self.save_template("THAT"), self.save_template("THIS"),
            self.save_template("ARG"), self.save_template("LCL"))
    }

    fn arithmetic_template_jmp(&mut self, jump_type: &str) -> String {
        self.jump_flag += 1;

        format!("@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@FALSE{1}\nD;\
                 {0}\n@SP\nA=M-1\nM=-1\n@CONTINUE{1}\n0;\
                 JMP\n(FALSE{1})\n@SP\nA=M-1\nM=0\n(CONTINUE{1})\n",
                jump_type,
                self.jump_flag - 1)
    }

    fn push_template(&mut self, seg: &str, index: i32, direct: bool) -> String {
        format!("@{}\nD=M\n{}@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                seg,
                if direct {
                    "".to_string()
                } else {
                    format!("@{}\nA=D+A\nD=M\n", index)
                })
    }

    fn pop_template(&mut self, seg: &str, index: i32, direct: bool) -> String {
        format!("@{0}\n{1}@R13\nM=D\n@SP\nAM=M-1\nD=M\n@R13\nA=M\nM=D\n",
                seg,
                if direct {
                    "D=A\n".to_string()
                } else {
                    format!("D=M\n@{}\nD=D+A\n", index)
                })
    }
}
