use bookmark_language::{
    instruction::{meta, mutating, pure, reading},
    instruction_list,
    program::ProgramBuilder,
    value::Value,
    variable,
};
use clap::Parser;
use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Parser)]
struct Cli {
    file: Option<PathBuf>,
}

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Lang(#[from] bookmark_language::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

fn main() -> Result<(), Error> {
    let Cli { file: file_path } = Cli::parse();

    let mut p_builder = ProgramBuilder::new();
    let mut v_builder = variable::MapBuilder::new();

    let sleep_duration = Value::Float(0.0);

    let a = v_builder.insert_rw(1.into());
    let b = v_builder.insert_rw(1.into());
    let l = v_builder.reserve_ro();

    let sleep_print_1 = instruction_list![
        pure::put(sleep_duration.clone()),
        pure::Sleep,
        pure::put(1),
        pure::Debug,
    ];

    p_builder.push_instruction(instruction_list![
        pure::put("starting seq"),
        pure::Debug,
        sleep_print_1.clone(),
        sleep_print_1,
        reading::Clone(l),
        meta::Perform(Value::None),
    ]);

    let loop_body = instruction_list![
        pure::put(sleep_duration),
        pure::Sleep,
        mutating::Take(a),
        reading::add_clone(b),
        pure::Debug,
        mutating::Swap(b),
        mutating::Swap(a),
        reading::Clone(l),
        meta::Perform(Value::None),
    ];

    v_builder.set(l, loop_body.into())?;
    p_builder.is_fallible(true);

    let program = p_builder.build(v_builder.build());

    if let Some(file_path) = file_path {
        serde_json::to_writer_pretty(BufWriter::new(File::create(file_path)?), &program)?;
    } else {
        program.run(Value::None).map(|_| ())?;
    }
    Ok(())
}
