use core::{ffi, panic};
use regex;
use std::collections::HashMap;
mod libxc;
// use libxc::ffi_xc::{xc_number_of_functionals};

fn main() {
    // println!("Hello, world!");
    // get a string from command line
    let args: Vec<String> = std::env::args().collect();
    let name = &args[1];
    println!("parsing xc: {}", name);
    parse(name);
}

pub struct DFAComponent {
    pub factor: f64,
    pub func: String,
    pub param_positional: Vec<f64>,
    pub param_keyword: HashMap<String, f64>,
}

const WHITELIST_NONDFA:[&str;2] = ["MP2", "HF"];

impl DFAComponent {
    pub fn formatted_output(&self) -> String {
        // output line by line
        let mut result = format!("Factor: {}, Func: {}", self.factor, self.func);
        if !self.param_positional.is_empty() {
            result.push_str(&format!(", Positional Params: {:?}", self.param_positional));
        }
        if !self.param_keyword.is_empty() {
            result.push_str(&format!(", Keyword Params: {:?}", self.param_keyword));
        }
        result
    }

    pub fn to_valid_name(&mut self, functype: &str) -> &mut Self {
        // check whitelist
        if WHITELIST_NONDFA.contains(&self.func.as_str()) {
            return self;
        }
        // check alias
        // todo!();
        // search libxc full name
        // todo!();
        return self;
    }
    
}

pub fn get_available_functionals() {
    let n = unsafe{libxc::ffi_xc::xc_number_of_functionals()};
}

pub fn parse(xc: &str) {
    get_available_functionals();
    let (xc_pass1, captures) = parse_pass1(xc);
    let parts = parse_pass2(&xc_pass1);
    if parts.len() == 2 {
        let (xfac, xfuncs, xparams) = parse_pass3(parts[0], &captures);
        let mut x_components:Vec<DFAComponent> = to_dfa_component_raw(xfac, xfuncs, xparams);
        for comp in x_components.iter_mut() {
            comp.to_valid_name("X");
        }
        let (cfac, cfuncs, cparams) = parse_pass3(parts[1], &captures);
        let mut c_components:Vec<DFAComponent> = to_dfa_component_raw(cfac, cfuncs, cparams);
        for comp in c_components.iter_mut() {
            comp.to_valid_name("C");
        }
        x_components.iter().for_each(|c| {
            println!("X component: {}", c.formatted_output());
        });
        c_components.iter().for_each(|c| {
            println!("C component: {}", c.formatted_output());
        });
    } else {
        let (xcfac, xcfuncs, xcparams) = parse_pass3(parts[0], &captures);
        let mut xc_components:Vec<DFAComponent> = to_dfa_component_raw(xcfac, xcfuncs, xcparams);
        for comp in xc_components.iter_mut() {
            comp.to_valid_name("XC");
        }
        xc_components.iter().for_each(|c| {
            println!("XC component: {}", c.formatted_output());
        });
    }
}

pub fn parse_pass1(xc: &str) -> (String, Vec<String>) {
    // find "( )" in xc with regex and substitute with ":n"
    // record the content in "( )" in a vector
    let mut result = String::new();
    let re = regex::Regex::new(r"\((.*?)\)").unwrap();
    let mut count = 1;
    let mut last_index = 0;
    let mut captures = Vec::new();
    for cap in re.captures_iter(xc) {
        let m = cap.get(0).unwrap();
        result.push_str(&xc[last_index..m.start()]);
        result.push_str(&format!(":{}", count));
        captures.push(cap[1].to_string());
        count += 1;
        last_index = m.end();
    }
    result.push_str(&xc[last_index..]);
    // println!("pass1");
    // println!("xc: {}", result);
    // println!("captured params: {:?}", captures);

    (result, captures)
}

pub fn parse_pass2(xc: &str) -> Vec<&str> {
    // xc: "0.5*K1 + 0.5*K2:1, K3"
    // split by ","
    // panic if more than one ","
    let parts: Vec<&str> = xc.split(',').collect();
    if parts.len() > 2 {
        panic!("Error: more than one ',' in xc");
    }
    // println!("pass2");
    // println!("parts: {:?}", parts);
    parts
}

pub fn parse_pass3(xc: &str, param_captures: &Vec<String>) -> (Vec<f64>, Vec<String>, Vec<String>) {
    // xc: "21.5*K1 + 0.56*K2:1 + K3"
    // parsed to fac=[0.5, 0.5, 1.0], funcs=["K1", "K2:1", "K3"]
    let mut fac = Vec::new();
    let mut funcs = Vec::new();
    // let re = regex::Regex::new("([+-]?\\d*\\.?\\d*)\\*([A-Za-z0-9_:]+)").unwrap();
    let re = regex::Regex::new(r"([+-]?\d*\.?\d*)\*?([A-Za-z0-9_:]+)").unwrap();
    for cap in re.captures_iter(xc) {
        let factor = if &cap[1] == "" || &cap[1] == "+" {
            1.0
        } else if &cap[1] == "-" {
            -1.0
        } else {
            cap[1].parse::<f64>().unwrap()
        };
        fac.push(factor);
        funcs.push(cap[2].to_string());
    }
    // println!("pass3");
    // println!("factors: {:?}", fac);
    // println!("funcs: {:?}", funcs);

    // fac=[0.5, 0.5, 1.0], funcs=["K1", "K2:1", "K3"], param_captures=["x=1,y=2"]
    // remove ":n" in funcs and create a new vector ["", "x=1,y=2", ""]
    let mut params = Vec::new();
    for func in funcs.iter_mut() {
        if func.contains(":") {
            if let Some((key,index)) = func.split_once(":") {
                let index: usize = index.parse().unwrap();
                if index == 0 || index > param_captures.len() {
                    panic!("Error: index out of range in func {}", func);
                }
                params.push(param_captures[index - 1].clone());
                *func = key.to_string();
            } else {
                panic!("Error: invalid func format {}", func);
            }
        } else {
            params.push(String::new());
        }
    }
    // println!("pass3.1");
    // println!("factors: {:?}", fac);
    // println!("funcs: {:?}", funcs);
    // println!("params: {:?}", params);

    (fac, funcs, params)
            
}

pub fn to_dfa_component_raw(fac: Vec<f64>, funcs: Vec<String>, params: Vec<String>) -> Vec<DFAComponent> {
    if fac.len() != funcs.len() || fac.len() != params.len() {
        panic!("Error: length of fac, funcs, params do not match");
    }
    let mut components = Vec::new();
    for i in 0..fac.len() {
        let (param_keyword, param_positional) = parse_arguments(&params[i]);
        let component = DFAComponent {
            factor: fac[i],
            func: funcs[i].clone(),
            param_positional,
            param_keyword,
        };
        components.push(component);
    }
    components
}

fn parse_arguments(input: &str) -> (HashMap<String, f64>, Vec<f64>) {
    let re = regex::Regex::new(r"(\w+\s*=\s*[^,]+)|([^,]+)").unwrap();
    
    let mut keyword_args = HashMap::new();
    let mut positional_args = Vec::new();
    let mut has_keyword = false;
    let mut has_positional = false;

    for cap in re.captures_iter(input) {
        if let Some(keyword_match) = cap.get(1) {
            // Parse keyword argument
            let part = keyword_match.as_str().trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().parse::<f64>().unwrap();
                keyword_args.insert(key, value);
                has_keyword = true;
            }
        } else if let Some(positional_match) = cap.get(2) {
            let value = positional_match.as_str().trim().parse::<f64>().unwrap();
            positional_args.push(value);
            has_positional = true;
        }

        // Check for mixed usage and panic if detected
        if has_keyword && has_positional {
            panic!("Mixed usage of keyword and positional arguments detected in: '{}'", input);
        }
    }

    (keyword_args, positional_args)
}