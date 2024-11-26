use clap::Parser as ClapParser;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BytesType {
    pub maxLength: Option<u64>,
    pub minLength: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArrayType {
    items: Box<LexiconData>,
    pub maxLength: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordType {
    key: String,
    record: ObjectType,
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
    parameters: Option<ObjectType>,
    message: Option<SubscribeMessage>,
    errors: Vec<SomeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OutputType {
    description: Option<String>,
    encoding: String,
    #[serde(default)]
    schema: Option<Box<LexiconData>>, // FIXME: spec says it's object, but then says it is object, a ref or union of refs...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryType {
    #[serde(default)]
    parameters: Option<ObjectType>,
    output: Option<OutputType>,
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
    parameters: Option<ObjectType>,
    #[serde(default)]
    input: Option<InputType>,
    output: Option<OutputType>,
    #[serde(default)]
    errors: Vec<SomeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscribeMessage {
    schema: Box<LexiconData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamsType {
    required: Vec<String>,
    properties: LinkedHashMap<String, LexiconData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnionType {
    refs: Vec<String>,
    closed: Option<bool>,
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
    maxLength: Option<i64>,
    minLength: Option<i64>,
    maxGraphemes: Option<i64>,
    minGraphemes: Option<i64>,
    knownValues: Option<Vec<String>>,
    #[serde(default, rename = "enum")]
    allowed_enum: Option<Vec<String>>,
    default: Option<String>,
    #[serde(rename = "const")]
    constant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BooleanType {
    default: Option<bool>,
    #[serde(rename = "const")]
    constant: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntegerType {
    minimum: Option<i64>,
    maximum: Option<i64>,
    #[serde(default, rename = "enum")]
    allowed_enum: Option<Vec<i64>>,
    default: Option<i64>,
    #[serde(rename = "const")]
    constant: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefType {
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlobType {
    accept: Vec<String>,
    maxSize: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum LexiconDataType {
    String(StringType),
    Bytes(BytesType),
    Blob(BlobType),
    Array(ArrayType),
    Record(RecordType),
    Subscription(SubscriptionType),
    Query(QueryType),
    Procedure(ProcedureType),
    Params(ParamsType),
    Union(UnionType),
    Object(ObjectType),
    Ref(RefType),
    Unknown,
    Boolean(BooleanType),
    Null,
    #[serde(rename = "cid-link")]
    CidLink,
    Integer(IntegerType),
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
    pub revision: Option<String>,
    pub description: Option<String>,
    pub defs: LinkedHashMap<String, LexiconData>,
}

fn codegen_one_def(defname: &str, def: &LexiconData) -> String {
    match &def.data {
        LexiconDataType::Object(o) => {
            let mut fields_str = String::new();
            for (propname, propdef) in &o.properties {
                let is_required = o.required.contains(propname);
                let is_nullable = o.nullable.contains(propname);
                // Determine the Rust type based on the property definition
                let rust_type = match &propdef.data {
                    LexiconDataType::String(_) => "String".to_string(),
                    LexiconDataType::Integer(_) => "i64".to_string(),
                    LexiconDataType::Boolean(_) => "bool".to_string(),
                    LexiconDataType::Array(arr) => {
                        let inner_type = match &arr.items.data {
                            LexiconDataType::String(_) => "String".to_string(),
                            LexiconDataType::Integer(_) => "i64".to_string(),
                            LexiconDataType::Boolean(_) => "bool".to_string(),
                            LexiconDataType::Ref(r) => r.reference.split("#").last().unwrap_or(&r.reference).to_string(),
                            _ => "String".to_string() // Default fallback
                        };
                        format!("Vec<{}>", inner_type)
                    },
                    LexiconDataType::Ref(r) => r.reference.split("#").last().unwrap_or(&r.reference).to_string(),
                    LexiconDataType::CidLink => "String".to_string(),
                    LexiconDataType::Bytes(_) => "Vec<u8>".to_string(),
                    LexiconDataType::Object(inner_obj) => {
                        // For nested objects, we'll create a new type name based on the parent and property name
                        format!("{}{}", defname, propname.chars().next().unwrap().to_uppercase().collect::<String>() + &propname[1..])
                    },
                    _ => "String".to_string() // Default fallback
                };

                // Build the type with Option wrapper if needed
                let final_type = if !is_required || is_nullable {
                    format!("Option<{}>", rust_type)
                } else {
                    rust_type
                };

                // Add serde rename if the property name isn't valid Rust
                let rust_safe_name = if propname.contains('-') || propname.contains('.') {
                    format!("    #[serde(rename = \"{}\")]\n", propname)
                } else {
                    "".to_string()
                };

                // Add the field with its documentation if available
                if let Some(desc) = &propdef.description {
                    fields_str.push_str(&format!("    /// {}\n", desc));
                }
                fields_str.push_str(&rust_safe_name);

                // Convert property name to valid Rust identifier
                let rust_field_name = propname.replace('-', "_").replace('.', "_");
                fields_str.push_str(&format!("    pub {}: {},\n", rust_field_name, final_type));
            }

            // Generate the struct definition with derive macros
            format!(
                "#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct {} {{\n{}}}\n\n",
                defname,
                fields_str
            )
        },
        LexiconDataType::Union(u) => {
            let mut variants = String::new();
            for reference in &u.refs {
                let variant_name = reference.split('#').last().unwrap_or(reference);
                variants.push_str(&format!("    {},\n", variant_name));
            }
            format!(
                "#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(tag = \"type\")]\npub enum {} {{\n{}}}\n\n",
                defname,
                variants
            )
        },
        x => {
            format!("/* {}: {:#?} - not generated */\n", defname, x)
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
