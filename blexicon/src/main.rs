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
    items: Box<Data>,
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
    parameters: Box<Data>,
    message: SubscribeMessage,
    errors: Vec<SomeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OutputType {
    encoding: String,
    #[serde(default)]
    schema: Option<Box<Data>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryType {
    #[serde(default)]
    parameters: Option<Box<Data>>,
    output: OutputType,
    #[serde(default)]
    errors: Vec<SomeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputType {
    encoding: String,
    #[serde(default)]
    schema: Option<Box<Data>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcedureType {
    #[serde(default)]
    input: Option<InputType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscribeMessage {
    schema: Box<Data>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamsType {
    properties: LinkedHashMap<String, Data>,
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
    properties: LinkedHashMap<String, Data>,
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
enum DataType {
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
struct Data {
    #[serde(flatten)]
    data: DataType,
    description: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LexiconTypeDef {}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LexiconFile {
    pub lexicon: u32,
    pub id: String,
    pub defs: LinkedHashMap<String, Data>,
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
            println!("read: {:#?}", &lex);
        } else {
            panic!("Could not read {}", &fname);
        }
    }

    println!("Hello, here is your options: {:#?}", &opts);
}
