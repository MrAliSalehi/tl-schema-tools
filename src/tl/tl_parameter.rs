use serde::{Deserialize, Serialize};
use crate::models::responses::Diff;

pub fn parse_parameter(text: &str) -> (String, Vec<TlParameter>) {
    if text.trim().is_empty() { return (String::default(), vec![]); }

    let mut spl = text.split_whitespace();
    let id = spl.next().unwrap().to_string(); //38fe25b7
    let params = spl.collect::<Vec<_>>();
    if params.is_empty() { return (id, vec![]); }
    let mut tl_params = vec![];
    for param in params {
        let mut spl = param.split(":");
        let param_name = spl.next().unwrap();
        let param_type = spl.next().unwrap();
        let is_generic = param_type == "!X";
        let is_flag_placeholder = param_type == "#";
        if is_flag_placeholder {
            tl_params.push(TlParameter::flag_placeholder(param_name));
            continue;
        }

        if is_generic {
            tl_params.push(TlParameter::generic(param_name));
            continue;
        }

        let inner_type = if param_type.contains('<') {
            Some(param_type.split("<").last().unwrap().split(">").next().unwrap().to_owned())
        } else { None };

        if param_type.starts_with("flags") {
            let mut spl_dot = param_type.split(".");
            let flag_name = spl_dot.next().unwrap().to_string();
            if let Some(spl_q) = spl_dot.next() {
                let mut spl_q = spl_q.split("?");
                let flag_offset = spl_q.next().unwrap().to_string();
                let flag_param_type = spl_q.next().unwrap().to_string();
                let inner_type = if flag_param_type.contains('<') {
                    Some(flag_param_type.split("<").last().unwrap().split(">").next().unwrap().to_owned())
                } else { None };
                tl_params.push(TlParameter {
                    is_generic: false,
                    flag_name: Some(flag_name),
                    flag_offset: Some(flag_offset),
                    _type: flag_param_type,
                    name: param_name.to_string(),
                    is_flag_placeholder: false,
                    is_optional: true,
                    inner_type
                });
                continue;
            };
        }
        tl_params.push(TlParameter {
            is_generic: false,
            flag_offset: None,
            flag_name: None,
            name: param_name.to_string(),
            _type: param_type.to_string(),
            is_optional: false,
            is_flag_placeholder: false,
            inner_type
        })
    }


    (id, tl_params)
}

/// parameter of a constructor
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TlParameter {
    pub name: String,
    pub _type: String,
    pub flag_name: Option<String>,
    pub flag_offset: Option<String>,
    pub inner_type:Option<String>,
    pub is_generic: bool,
    pub is_optional: bool,
    pub is_flag_placeholder: bool,
}

impl TlParameter {
    pub fn diff<'a>(b: &'a Self, a: &'a Self) -> Option<Vec<Diff<'a>>> {
        let mut diffs = vec![];

        if a.name != b.name {
            diffs.push(Diff { from: a.name.to_owned(), to: b.name.to_owned(), field_name: "name" });
        }

        if a._type != b._type {
            diffs.push(Diff { from: a._type.to_owned(), to: b._type.to_owned(), field_name: "_type" });
        }

        if a.inner_type != b.inner_type {
            diffs.push(Diff {
                from: a.inner_type.to_owned().unwrap_or_default(),
                to: b.inner_type.to_owned().unwrap_or_default(),
                field_name: "inner_type",
            });
        }

        if a.flag_name != b.flag_name {
            diffs.push(Diff {
                from: a.flag_name.to_owned().unwrap_or_default(),
                to: b.flag_name.to_owned().unwrap_or_default(),
                field_name: "flag_name",
            });
        }

        if a.flag_offset != b.flag_offset {
            diffs.push(Diff {
                from: a.flag_offset.to_owned().unwrap_or_default(),
                to: b.flag_offset.to_owned().unwrap_or_default(),
                field_name: "flag_offset",
            });
        }

        if a.is_generic != b.is_generic {
            let from = if a.is_generic { "true" } else { "false" }.to_owned();
            let to = if b.is_generic { "true" } else { "false" }.to_owned();
            diffs.push(Diff { from, to, field_name: "is_generic" });
        }

        if a.is_optional != b.is_optional {
            let from = if a.is_optional { "true" } else { "false" }.to_owned();
            let to = if b.is_optional { "true" } else { "false" }.to_owned();
            diffs.push(Diff { from, to, field_name: "is_optional" });
        }

        if a.is_flag_placeholder != b.is_flag_placeholder {
            let from = if a.is_flag_placeholder { "true" } else { "false" }.to_owned();
            let to = if b.is_flag_placeholder { "true" } else { "false" }.to_owned();
            diffs.push(Diff { from, to, field_name: "is_flag_placeholder" });
        }

        if diffs.is_empty() {
            None
        } else {
            Some(diffs)
        }
    }
    pub fn flag_placeholder(flag_name: &str) -> Self {
        Self {
            flag_name: None,
            flag_offset: None,
            is_flag_placeholder: true,
            is_optional: false,
            is_generic: false,
            _type: String::default(),
            name: flag_name.to_string(),
            inner_type:None
        }
    }
    pub fn generic(param_name: &str) -> TlParameter {
        Self {
            flag_name: None,
            flag_offset: None,
            is_flag_placeholder: true,
            is_optional: false,
            is_generic: true,
            _type: String::default(),
            name: param_name.to_string(),
            inner_type:None
        }
    }
}