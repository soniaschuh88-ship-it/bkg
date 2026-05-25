use std::path::PathBuf;
use serde::{Deserialize,Serialize};
use bkg_core::{LogicalTimestamp,RealmId};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WriteOp{pub timestamp:LogicalTimestamp,pub realm:RealmId,pub label:String,#[serde(default,skip_serializing_if="Option::is_none")]pub payload:Option<serde_json::Value>,#[serde(default,skip_serializing_if="Option::is_none")]pub file_path:Option<PathBuf>,#[serde(default,skip_serializing_if="Option::is_none")]pub data_hash:Option<String>}
impl WriteOp{pub fn new(t:LogicalTimestamp,r:RealmId,l:impl Into<String>)->Self{Self{timestamp:t,realm:r,label:l.into(),payload:None,file_path:None,data_hash:None}}pub fn with_file(mut self,p:PathBuf)->Self{self.file_path=Some(p);self}}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReadOp{pub timestamp:LogicalTimestamp,pub realm:RealmId,pub label:String,pub target:String,#[serde(default,skip_serializing_if="Option::is_none")]pub data_hash:Option<String>}
impl ReadOp{pub fn new(t:LogicalTimestamp,r:RealmId,l:impl Into<String>,tgt:impl Into<String>)->Self{Self{timestamp:t,realm:r,label:l.into(),target:tgt.into(),data_hash:None}}}
