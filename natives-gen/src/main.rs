use std::collections::HashMap;
use std::io::*;

use full_moon::ast::{
    Call, Expression, Field, FunctionArgs, FunctionCall, Prefix, Stmt, Suffix, Value,
};

// TODO: client / server / shared
//       group by namespaces
fn main() {
    let params =
        std::fs::read_to_string("E:/sources/c/fivem-fork/ext/natives/inp/natives_cfx.lua").unwrap();

    let types =
        std::fs::read_to_string("E:/sources/c/fivem-fork/ext/natives/codegen_types.lua").unwrap();

    let ast_params = full_moon::parse(&params).unwrap();
    let ast_types = full_moon::parse(&types).unwrap();

    let params: Vec<NativeParam> = ast_params
        .nodes()
        .stmts()
        .filter_map(|stmt| unwrap(stmt))
        .collect();

    let types: Vec<NativeParam> = ast_types
        .nodes()
        .stmts()
        .filter_map(|stmt| unwrap(stmt))
        .collect();

    let types = format_types(types);
    let natives = format_natives(params, types);

    let mut file = std::fs::File::create("natives.rs").unwrap();

    for native in natives.iter() {
        writeln!(file, "{}", make_native(native));
    }
}

#[derive(Debug)]
enum ApiSet {
    Server,
    Client,
    Shared,
}

impl From<String> for ApiSet {
    fn from(text: String) -> Self {
        match text.as_str() {
            "client" => Self::Client,
            "shared" => Self::Shared,
            _ => Self::Server,
        }
    }
}

impl Default for ApiSet {
    fn default() -> Self {
        ApiSet::Server
    }
}

#[derive(Debug, Default)]
struct Native {
    name: String,
    jhash: u64,
    apiset: ApiSet,
    namespace: Option<String>,
    game: Option<String>,
    arguments: Vec<RustArgument>,
    returns: Option<RustType>,
}

#[derive(Debug)]
struct NativeParam {
    name: String,
    argument: Argument,
}

#[derive(Debug, Default)]
struct CfxType {
    name: String,
    native_type: String,
}

#[derive(Debug)]
struct RustType {
    name: String,
    convert: Option<String>,
}

#[derive(Debug)]
struct RustArgument {
    name: String,
    is_ptr: bool,
    ty: RustType,
}

#[derive(Debug)]
enum Argument {
    String(String),
    Table(Vec<(String, Vec<Box<Argument>>)>),
}

impl Argument {
    fn to_string(&self) -> String {
        match self {
            Argument::String(str) => str.clone(),
            Argument::Table(_) => "table".to_owned(),
        }
    }
}

fn unwrap_name(call: &FunctionCall) -> Option<String> {
    match call.prefix() {
        Prefix::Name(name) => Some(name.token().to_string()),
        _ => None,
    }
}

fn unwrap_argument(suffix: Option<&Suffix>) -> Option<Argument> {
    let suffix = suffix?;

    match suffix {
        Suffix::Call(call) => match call {
            Call::AnonymousCall(args) => match args {
                FunctionArgs::String(named) => match named.token_type() {
                    full_moon::tokenizer::TokenType::StringLiteral { literal, .. } => {
                        Some(Argument::String(literal.to_string()))
                    }

                    _ => None,
                },
                FunctionArgs::TableConstructor(table) => {
                    let table = table
                        .fields()
                        .iter()
                        .filter_map(|field| match field {
                            Field::NoKey(expr) => match expr {
                                Expression::Value { value } => match &**value {
                                    Value::FunctionCall(call) => {
                                        let name = unwrap_name(call)?;
                                        let args = call
                                            .suffixes()
                                            .filter_map(|suf| unwrap_argument(Some(suf)))
                                            .map(|arg| Box::new(arg))
                                            .collect();

                                        Some((name, args))
                                    }

                                    _ => None,
                                },
                                _ => None,
                            },

                            _ => None,
                        })
                        .collect();

                    Some(Argument::Table(table))
                }

                _ => None,
            },

            _ => None,
        },

        _ => None,
    }
}

fn unwrap(stmt: &Stmt) -> Option<NativeParam> {
    match stmt {
        Stmt::FunctionCall(call) => {
            let mut suffixes = call.suffixes();
            let name = unwrap_name(call)?;
            let argument = unwrap_argument(suffixes.next())?;

            Some(NativeParam { name, argument })
        }

        _ => None,
    }
}

fn convert_type(ty: &CfxType, in_ret: bool) -> RustType {
    match ty.native_type.as_str() {
        "string" => {
            if in_ret {
                RustType {
                    name: "String".to_owned(),
                    convert: None,
                }
            } else {
                RustType {
                    name: "impl fivem_core::types::AsCharPtr".to_owned(),
                    convert: Some("as_char_ptr().into()".to_owned()),
                }
            }
        }

        "int" => RustType {
            name: "u32".to_owned(),
            convert: None,
        },

        "long" => RustType {
            name: "u64".to_owned(),
            convert: None,
        },

        "float" => RustType {
            name: "f32".to_owned(),
            convert: None,
        },

        "vector3" => RustType {
            name: "fivem_core::types::Vector3".to_owned(),
            convert: None,
        },

        "func" => {
            if in_ret {
                RustType {
                    name: "fivem_core::ref_funcs::ExternRefFunction".to_owned(),
                    convert: None,
                }
            } else {
                RustType {
                    name: "fivem_core::ref_funcs::RefFunction".to_owned(),
                    convert: None,
                }
            }
        }

        "object" => {
            if in_ret {
                RustType {
                    name: "fivem_core::types::Packed<Ret>".to_owned(),
                    convert: None,
                }
            } else {
                RustType {
                    name: "impl serde::Serialize".to_owned(),
                    convert: None,
                }
            }
        }

        "bool" => RustType {
            name: "bool".to_owned(),
            convert: None,
        },

        _ => RustType {
            name: "()".to_owned(),
            convert: None,
        },
    }
}

fn find_type<'a>(map: &'a HashMap<String, CfxType>, name: &str) -> Option<(bool, &'a CfxType)> {
    if name != "charPtr" && name.ends_with("Ptr") {
        let fixed = name.strip_suffix("Ptr")?;
        Some((true, map.get(fixed)?))
    } else {
        Some((false, map.get(name)?))
    }
}

fn format_types(types: Vec<NativeParam>) -> HashMap<String, CfxType> {
    let mut types = types.iter();
    let mut cfx_type: Option<CfxType> = None;
    let mut formated = HashMap::new();

    loop {
        if let Some(ty) = types.next() {
            if ty.name == "type" {
                if let Some(cfx_type) = cfx_type.take() {
                    formated.insert(cfx_type.name.clone(), cfx_type);
                }

                if let Argument::String(str) = &ty.argument {
                    println!("{}", str);
                    cfx_type = Some(CfxType {
                        name: str.to_owned(),
                        ..Default::default()
                    });
                }

                continue;
            } else {
                if let Some(cfx_type) = cfx_type.as_mut() {
                    match ty.name.as_str() {
                        "nativeType" => {
                            if cfx_type.native_type.len() == 0 {
                                cfx_type.native_type = ty.argument.to_string().to_ascii_lowercase();
                            }
                        }

                        "subType" => {
                            cfx_type.native_type = ty.argument.to_string().to_ascii_lowercase()
                        }

                        _ => (),
                    }
                }
            }
        } else {
            if let Some(cfx_type) = cfx_type.take() {
                formated.insert(cfx_type.name.clone(), cfx_type);
            }

            break;
        }
    }

    formated
}

fn format_natives(params: Vec<NativeParam>, types: HashMap<String, CfxType>) -> Vec<Native> {
    let mut params = params.iter();
    let mut native: Option<Native> = None;
    let mut natives = vec![];

    loop {
        if let Some(param) = params.next() {
            if param.name == "native" {
                if let Some(mut native) = native.take() {
                    if native.jhash == 0 {
                        native.jhash = joaat::hash_ascii_lowercase(
                            native.name.to_ascii_lowercase().as_bytes(),
                        ) as u64;
                    }

                    natives.push(native);
                }

                if let Argument::String(str) = &param.argument {
                    native = Some(Native {
                        name: str.to_owned(),
                        ..Default::default()
                    });
                }

                continue;
            } else {
                if let Some(native) = native.as_mut() {
                    let arg = param.argument.to_string();

                    match param.name.as_str() {
                        "jhash" => (),
                        "apiset" => native.apiset = ApiSet::from(arg),
                        "ns" => native.namespace = Some(arg),
                        "game" => native.game = Some(arg),
                        "returns" => {
                            native.returns =
                                find_type(&types, &arg).map(|(_, ty)| convert_type(ty, true));
                        }

                        "arguments" => {
                            if let Argument::Table(args) = &param.argument {
                                native.arguments = args
                                    .iter()
                                    .filter_map(|(ty, name)| {
                                        let (is_ptr, ty) = find_type(&types, ty)?;
                                        let ty = convert_type(ty, false);

                                        Some(RustArgument {
                                            name: name.get(0)?.to_string(),
                                            is_ptr,
                                            ty,
                                        })
                                    })
                                    .collect();
                            }
                        }

                        "doc" => (),
                        _ => (),
                    }
                }
            }
        } else {
            break;
        }
    }

    natives
}

fn make_native(native: &Native) -> String {
    let name = {
        let generics = if let Some(ret) = &native.returns {
            if ret.name.ends_with("<Ret>") {
                "<Ret: serde::de::DeserializeOwned>"
            } else {
                ""
            }
        } else {
            ""
        };

        format!("{}{}", native.name.to_ascii_lowercase(), generics)
    };

    let rettype = native
        .returns
        .as_ref()
        .map(|ret| ret.name.clone())
        .unwrap_or_else(|| "()".to_owned());

    let args = native
        .arguments
        .iter()
        .map(|arg| {
            format!(
                "_{}: {}{}",
                arg.name,
                if arg.is_ptr { "&mut " } else { "" },
                arg.ty.name
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    let body = {
        let args = native
            .arguments
            .iter()
            .map(|arg| {
                if let Some(conv) = &arg.ty.convert {
                    format!("_{}.{}", arg.name, conv)
                } else {
                    format!("_{}.into()", arg.name)
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        format!("fivem_core::invoker::invoke({}, &[{}])", native.jhash, args)
    };

    format!(
        "pub fn {}({}) -> Result<{}, fivem_core::invoker::InvokeError> {{ {} }}",
        name, args, rettype, body
    )
}
