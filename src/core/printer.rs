use super::vm::{
    RuntimeVariable, RuntimeVariableAbstractData, RuntimeVariableAbstractPrimitiveData,
};

pub struct Printer<'a> {
    indent_delimeter: &'a str,
    indent: usize,
}

impl<'a> Printer<'a> {
    pub fn new() -> Self {
        Printer {
            indent: 0,
            indent_delimeter: " ",
        }
    }

    pub fn do_print(&mut self, var: RuntimeVariable<'a>) -> String {
        let mut out = String::new();

        self.print_var(&mut out, &var.data);

        out
    }

    fn print_var(&mut self, out: &mut String, var: &RuntimeVariableAbstractData) {
        let indent = &format!("{}", self.indent_delimeter.repeat(self.indent))[..];

        match &var {
            RuntimeVariableAbstractData::None => {
                out.push_str("None");
            }
            RuntimeVariableAbstractData::Primitive(e) => match e {
                RuntimeVariableAbstractPrimitiveData::Article(e) => {
                    out.push_str(&e.title);
                }
                RuntimeVariableAbstractPrimitiveData::Category(e) => {
                    out.push_str(e);
                }
                RuntimeVariableAbstractPrimitiveData::Integer(e) => {
                    out.push_str(&e.to_string()[..]);
                }
                RuntimeVariableAbstractPrimitiveData::String(e) => {
                    out.push_str(e);
                }
            },
            RuntimeVariableAbstractData::Array(e) => {
                out.push_str(indent);
                out.push_str("[\n");
                self.indent += 4;

                let iindent = &format!("{}", self.indent_delimeter.repeat(self.indent))[..];

                for var in e.iter() {
                    out.push_str(iindent);
                    self.print_var(out, var);
                    out.push_str(",\n");
                }

                out.push_str(indent);
                out.push_str("]\n");
                self.indent -= 4;
            }
            RuntimeVariableAbstractData::Set(e) => {
                out.push_str(indent);
                out.push_str("(\n");
                self.indent += 4;

                let iindent = &format!("{}", self.indent_delimeter.repeat(self.indent))[..];

                for var in e.iter() {
                    out.push_str(iindent);
                    self.print_var(out, var);
                    out.push_str(",\n");
                }

                out.push_str(indent);
                out.push_str(")\n");
                self.indent -= 4;
            }
            RuntimeVariableAbstractData::Tuple(e) => {
                out.push_str("(");
                e.iter().enumerate().for_each(|(i, x)| {
                    self.print_var(out, x);

                    if i != e.len() - 1 {
                        out.push_str(", ");
                    }
                });
                out.push_str(")");
            }
        }
    }
}
