// use crate::libxc::ffi_xc;
use super::libxc;
// use libxc::ffi_xc::{xc_number_of_functionals};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ffi;
use serde_json;

#[derive(Clone,PartialEq,Debug)]
pub enum ComponentType {
    HF,
    PT2,
    RPA,
    Disp,
    Libxc,
    Unknown,
}

impl ComponentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentType::HF => "HF",
            ComponentType::PT2 => "PT2",
            ComponentType::RPA => "RPA",
            ComponentType::Disp => "Disp",
            ComponentType::Libxc => "Libxc",
            ComponentType::Unknown => "Unknown",
        }
    }
}

lazy_static! {
    pub static ref AVAIL_FUNC : HashMap<String, i32> = get_available_functionals();

    pub static ref CODES: HashMap<&'static str, i32> = HashMap::from([
        // LDA
        ("LDA", 1),
        ("SLATER", 1),
        ("VWN3", 8),
        ("VWNRPA", 8),
        ("VWN5", 7),
        // HYB_GGA
        ("PBE0", 406),
        ("B3LYPG", 402),
        ("X3LYPG", 411),
    ]);

    pub static ref ALIAS: HashMap<&'static str, &'static str> = HashMap::from([
        // LDA
        ("SVWN", "SLATER,VWN"),
        // GGA
        ("BLYP", "B88,LYP"),
        ("BP86", "B88,P86"),
        ("PW91", "PW91,PW91"),
        ("PBE", "PBE,PBE"),
        ("REVPBE", "PBE_R,PBE"),
        ("PBESOL", "PBE_SOL,PBE_SOL"),
        ("SOGGA", "SOGGA,PBE"),
        ("OLYP", "OPTX,LYP"),
        ("OPBE", "OPTX,PBE"),
        ("RPBE", "RPBE,PBE"),
        ("BPBE", "B88,PBE"),
        ("MPW91", "MPW91,PW91"),
        ("SOGGA11", "SOGGA11,SOGGA11"),
        ("KT1", "KT1,VWN"),
        ("KT2", "GGA_XC_KT2"),
        ("KT3", "GGA_XC_KT3"),
        ("GAM", "GAM,GAM"),
        ("N12", "N12,N12"),
        ("PBEOP", "PBE,OP_PBE"),
        ("BOP", "B88,OP_B88"),
        // MGGA
        ("PKZB", "PKZB,PKZB"),
        ("TPSS", "TPSS,TPSS"),
        ("REVTPSS", "REVTPSS,REVTPSS"),
        ("SCAN", "SCAN,SCAN"),
        ("RSCAN", "RSCAN,RSCAN"),
        ("R2SCAN", "R2SCAN,R2SCAN"),
        ("SCANL", "SCANL,SCANL"),
        ("R2SCANL", "R2SCANL,R2SCANL"),
        ("BLOC", "BLOC,TPSSLOC"),
        ("MS0", "MS0,REGTPSS"),
        ("MS1", "MS1,REGTPSS"),
        ("MS2", "MS2,REGTPSS"),
        ("MS2H", "MS2H,REGTPSS"),
        ("MVS", "MVS,REGTPSS"),
        ("MVSH", "MVSH,REGTPSS"),
        ("M06_L", "M06_L,M06_L"),
        ("M11_L", "M11_L,M11_L"),
        ("MN12_L", "MN12_L,MN12_L"),
        ("MN15_L", "MN15_L,MN15_L"),
        ("MBEEF", "MBEEF,PBE_SOL"),
        ("REVSCAN", "REVSCAN,REVSCAN"),
        ("REVSCAN_VV10", "REVSCAN,REVSCAN_VV10"),
        ("SCAN_VV10", "SCAN,SCAN_VV10"),
        ("SCAN_RVV10", "SCAN,SCAN_RVV10"),
        // HYB_LDA
        ("HFPW92", "HF,PW_MOD"),
        ("SPW92", "SLATER,PW_MOD"),
        // HYB_GGA
        ("HFLYP", "HF,LYP"),
        ("SOGGA11_X", "SOGGA11_X,SOGGA11_X"),
        ("N12_SX", "N12_SX,N12_SX"),
        // HYB_MGGA
        ("DLDF", "DLDF,DLDF"),
        ("M06_SX", "M06_SX,M06_SX"),
        ("MN12_SX", "MN12_SX,MN12_SX"),
        ("MN15", "MN15,MN15"),
        ("SCAN0", "SCAN0,SCAN"),
        ("M05", "M05,M05"),
        ("M06", "M06,M06"),
        ("M05_2X", "M05_2X,M05_2X"),
        ("M06_2X", "M06_2X,M06_2X"),
    ]);

    pub static ref WHITELIST_NONLIBXC:HashMap<&'static str, ComponentType> = HashMap::from([
        ("HF", ComponentType::HF),
        // todo SR_HF
        ("MP2", ComponentType::PT2),
    ]);

    pub static ref MULTISTEP:HashMap<String, XC2step> = load_json_functionals();

}

pub fn get_name(id: i32) -> String {
    let name = unsafe{
        let c_str = ffi::CStr::from_ptr(libxc::ffi_xc::xc_functional_get_name(id));
        c_str.to_str().unwrap().to_owned()
    };
    name.to_uppercase()
}

pub fn get_available_functionals() -> HashMap<String, i32> {
    let n = unsafe{libxc::ffi_xc::xc_number_of_functionals()};
    // println!("Number of functionals in libxc: {}", n);
    // empty vector with length n
    let mut ids:Vec<i32> = vec![0; n as usize];
    unsafe{
        libxc::ffi_xc::xc_available_functional_numbers(ids.as_mut_ptr());
    }
    // println!("{:?}", ids);
    let mut available_functionals = HashMap::new();
    for id in ids {
        let name = unsafe{
            let c_str = ffi::CStr::from_ptr(libxc::ffi_xc::xc_functional_get_name(id));
            c_str.to_str().unwrap().to_owned()
        };
        available_functionals.insert(name.to_uppercase(), id);
    }
    // println!("Available functionals: {:?}", available_functionals);
    available_functionals
}

#[derive(Clone,PartialEq,Debug)]
pub struct XC2step {
    pub code_scf: String,
    pub code: String,
    pub reference: String,
}

pub fn load_json_functionals() -> HashMap<String, XC2step> {
    let json_str = include_str!("./family_xdh.json");
    // load data from json_str, then construct HashMap<String, XC2step>
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let mut map = HashMap::new();
    if let serde_json::Value::Object(obj) = v {
        for (key, value) in obj {
            if let serde_json::Value::Object(inner_obj) = value {
                let code_scf = inner_obj.get("code_scf").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let code = inner_obj.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let reference = inner_obj.get("ref").and_then(|v| v.as_str()).unwrap_or("").to_string();
                map.insert(key, XC2step { code_scf, code, reference });
            }
        }
    }
    map
}