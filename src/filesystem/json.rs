use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use rmp_serde;
use crate::{false_if_err,err,err_plus};

use crate::common::ResourceHandler;

pub fn from_json<T: for <'a> Deserialize<'a>>(path: &PathBuf, resources: &mut ResourceHandler) -> Option<T> {
    let f = resources.eat(err_plus!(fs::read_to_string(&path), path))?;
    let v: T = resources.eat(err!(serde_json::from_str(&f)))?;
    return Some(v);
}

pub fn to_json<T: Serialize>(t: &T, path: &PathBuf, resources: &mut ResourceHandler) -> bool {
    let s = false_if_err!(serde_json::to_string_pretty(t), resources);
    false_if_err!(fs::write(path, s), resources);
    return true;
}

pub fn to_msgpack<T: Serialize>(t: &T, path: &PathBuf, resources: &mut ResourceHandler) -> bool {
    let s = false_if_err!(rmp_serde::encode::to_vec(t), resources);
    false_if_err!(fs::write(path, s), resources);
    return true;
}

pub fn from_msgpack<T: for <'a> Deserialize<'a>>(path: &PathBuf, resources: &mut ResourceHandler) -> Option<T> {
    let f = resources.eat(err_plus!(fs::read(&path), path))?;
    let v: T = resources.eat(err!(rmp_serde::decode::from_slice(&f)))?;
    return Some(v);
}
