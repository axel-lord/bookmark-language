use bookmark_language::{
    instruction::{Meta, Mutating, Pure},
    instruction_list,
    program::ProgramBuilder,
    value::Value,
    variable, Result,
};

fn main() -> Result<()> {
    let mut p_builder = ProgramBuilder::new();
    let mut v_builder = variable::MapBuilder::new();

    let sleep_duration = Value::Float(0.1);

    let a = v_builder.insert_rw(1.into());
    let b = v_builder.insert_rw(1.into());
    let t = v_builder.reserve_rw();
    let l = v_builder.reserve_ro();

    p_builder.push_instruction(instruction_list![
        Pure::value("starting seq"),
        Pure::Debug,
        Pure::Value(sleep_duration.clone()),
        Pure::Sleep,
        Pure::value(1),
        Pure::Debug,
        Pure::Value(sleep_duration.clone()),
        Pure::Sleep,
        Pure::value(1),
        Pure::Debug,
        Pure::Clone(l),
        Meta::Perform(Value::None),
    ]);

    let loop_body = instruction_list![
        Pure::Value(sleep_duration),
        Pure::Sleep,
        Mutating::Take(a),
        Pure::Add(Value::Id(b)),
        Pure::Debug,
        Mutating::Assign(t),
        Mutating::Take(b),
        Mutating::Assign(a),
        Mutating::Take(t),
        Mutating::Assign(b),
        Pure::Clone(l),
        Meta::Perform(Value::None),
    ];

    v_builder.set(l, loop_body.into())?;

    p_builder
        .build(v_builder.build())
        .run(Value::None)
        .map(|_| ())
}
