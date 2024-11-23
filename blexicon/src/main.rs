use clap::Parser as ClapParser;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BytesType {
    pub maxLength: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArrayType {
    items: Box<LexiconData>,
    pub maxLength: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeType {
    format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SomeError {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionType {
    parameters: Box<LexiconData>,
    message: SubscribeMessage,
    errors: Vec<SomeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OutputType {
    encoding: String,
    #[serde(default)]
    schema: Option<Box<LexiconData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryType {
    #[serde(default)]
    parameters: Option<Box<LexiconData>>,
    output: OutputType,
    #[serde(default)]
    errors: Vec<SomeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputType {
    encoding: String,
    #[serde(default)]
    schema: Option<Box<LexiconData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcedureType {
    #[serde(default)]
    input: Option<InputType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscribeMessage {
    schema: Box<LexiconData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamsType {
    properties: LinkedHashMap<String, LexiconData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnionType {
    refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObjectType {
    #[serde(default)]
    required: Vec<String>,
    #[serde(default)]
    nullable: Vec<String>,
    properties: LinkedHashMap<String, LexiconData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StringType {
    format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefType {
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum LexiconDataType {
    String(StringType),
    Bytes(BytesType),
    Array(ArrayType),
    Subscription(SubscriptionType),
    Query(QueryType),
    Procedure(ProcedureType),
    Params(ParamsType),
    Union(UnionType),
    Object(ObjectType),
    Ref(RefType),
    Unknown,
    Boolean,
    #[serde(rename = "cid-link")]
    CidLink,
    Integer,
    Token,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LexiconData {
    #[serde(flatten)]
    data: LexiconDataType,
    description: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LexiconTypeDef {}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LexiconFile {
    pub lexicon: u32,
    pub id: String,
    pub defs: LinkedHashMap<String, LexiconData>,
}

fn codegen_one_def(defname: &str, def: &LexiconData) -> String {
    match &def.data {
        LexiconDataType::Object(o) => {
            let mut fields_str = format!("");
            for (propname, propdef) in &o.properties {
                fields_str.push_str(&format!("   {},\n", propname));
            }
            format!(
                r##"struct {} {{
{}}}

"##,
                defname, fields_str
            )
        }
        x => {
            format!("/* {}: {:#?} - not generated */\n", &defname, &def)
        }
    }
    .to_string()
}

/// This program aims to compile a .json lexicon file into an Rust source code.
#[derive(Debug, Clone, ClapParser, Serialize, Deserialize)]
#[clap(version = "0.0.1", author = "Andrew Yourtchenko <ayourtch@gmail.com>")]
struct Opts {
    /// Target hostname to do things on
    #[clap()]
    source: Vec<String>,

    /// Override options from this yaml/json file
    #[clap(short, long)]
    options_override: Option<String>,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() {
    let opts: Opts = Opts::parse();

    // allow to load the options, so far there is no good built-in way
    let opts = if let Some(fname) = &opts.options_override {
        if let Ok(data) = std::fs::read_to_string(&fname) {
            let res = serde_json::from_str(&data);
            if res.is_ok() {
                res.unwrap()
            } else {
                serde_yaml::from_str(&data).unwrap()
            }
        } else {
            opts
        }
    } else {
        opts
    };

    if opts.verbose > 4 {
        let data = serde_json::to_string_pretty(&opts).unwrap();
        println!("{}", data);
        println!("===========");
        let data = serde_yaml::to_string(&opts).unwrap();
        println!("{}", data);
    }

    for fname in &opts.source {
        println!("Reading {}", &fname);
        if let Ok(data) = std::fs::read_to_string(&fname) {
            let lex: LexiconFile = serde_json::from_str(&data).unwrap();
            // println!("read: {:#?}", &lex);
            for (name, def) in &lex.defs {
                println!("{}", codegen_one_def(name, def));
            }
        } else {
            panic!("Could not read {}", &fname);
        }
    }
}
