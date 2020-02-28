use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "prog.pest"]
struct ProgParser;

#[derive(Debug, Clone)]
enum StmtType {
    Print { text: String },
    FnBegin { name: String, offset_to_end: usize },
    FnEnd,
    Call { name: String },
}

#[derive(Debug, Clone)]
struct Program {
    stmts: Vec<StmtType>,
    fns: HashMap<String, usize>,
}

fn parse_stmts(s: String) -> Option<Program> {
    let stmts = ProgParser::parse(Rule::Prog, &s);
    if let Err(e) = stmts {
        eprintln!("{}", e);
        return None;
    }
    let stmts = stmts.unwrap().next().unwrap().into_inner();

    let mut stmt_list = vec![];
    let mut fns = HashMap::new();
    let mut fn_start = None;
    //dbg!(&stmts);
    for stmt in stmts {
        //dbg!(&stmt);
        match stmt.as_rule() {
            Rule::Print => stmt_list.push(StmtType::Print {
                text: stmt
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .as_str()
                    .to_owned(),
            }),
            Rule::FnBegin => {
                if fn_start.is_some() {
                    eprintln!("You cannot nest FnBegin.");
                    std::process::exit(1);
                }
                let fn_name = stmt
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .as_str()
                    .to_owned();
                fn_start = Some(stmt_list.len());
                fns.insert(fn_name.clone(), stmt_list.len());
                stmt_list.push(StmtType::FnBegin {
                    name: fn_name,
                    offset_to_end: 0,
                });
            }
            Rule::FnEnd => {
                if fn_start.is_none() {
                    eprintln!("A stray FnEnd detected.");
                    std::process::exit(1);
                }
                let start = fn_start.take().unwrap();
                if let StmtType::FnBegin { ref name, .. } = stmt_list[start] {
                    stmt_list[start] = StmtType::FnBegin {
                        name: name.clone(),
                        offset_to_end: stmt_list.len() - start,
                    };
                } else {
                    unreachable!();
                }
                stmt_list.push(StmtType::FnEnd);
            }
            Rule::Call => stmt_list.push(StmtType::Call {
                name: stmt
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .as_str()
                    .to_owned(),
            }),
            _ => unreachable!(),
        }
    }
    Some(Program {
        stmts: stmt_list,
        fns,
    })
}

pub fn wait_keypress() {
    use crossterm::event::*;
    loop {
        if let Event::Key(KeyEvent { .. }) = read().unwrap() {
            return;
        }
    }
}

fn process_stmts(prog: Program) {
    use std::io::Write;
    let mut ret_idx = None;
    let mut i = 0;
    while i < prog.stmts.len() {
        //dbg!(i);
        //dbg!(&prog.stmts[i]);
        match &prog.stmts[i] {
            StmtType::Print { text } => {
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::style::Print(format!(
                        "{:04} : {}\r\n[Proceed with any key]\r",
                        i, text
                    ))
                );
                let _ = wait_keypress();
                i += 1;
            }
            StmtType::FnBegin { offset_to_end, .. } => {
                i += offset_to_end + 1;
            }
            StmtType::Call { name } => {
                if let Some(idx) = prog.fns.get(name) {
                    ret_idx = Some(i+1);
                    i = *idx + 1;
                } else {
                    unreachable!()
                }
            }
            StmtType::FnEnd => {
                if ret_idx.is_some() {
                    i = ret_idx.take().unwrap();
                } else {
                    unreachable!()
                }
            }
            _ => unreachable!(),
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("filename was not provided");
        std::process::exit(1);
    });
    let s = std::fs::read_to_string(path).unwrap();

    eprint!("Loading the file...");
    let parsed = parse_stmts(s);
    eprintln!(" completed");

    if let Some(i) = parsed {
        ctrlc::set_handler(|| {
            let _ = crossterm::terminal::disable_raw_mode();
        })
        .unwrap();
        let _ = crossterm::terminal::enable_raw_mode();
        process_stmts(i);
        let _ = crossterm::terminal::disable_raw_mode();
    } else {
        std::process::exit(1);
    }
}
