use core::{panic};
use regex;
use std::collections::HashMap;
use lazy_static::lazy_static;
mod libxc;
mod xc_helper;
use xc_helper::{AVAIL_FUNC, ALIAS, CODES, WHITELIST_NONLIBXC, ComponentType, 
    get_name, MULTISTEP, XC2step};

fn main() {
    // println!("Hello, world!");
    // get a string from command line
    let args: Vec<String> = std::env::args().collect();
    let name = &args[1];
    // println!("parsing xc: {}", name);
    let final_results = parse(name);
}



#[derive(Clone)]
pub struct DFAComponent {
    pub factor: f64,
    pub func: String,
    pub func_full_name: String,
    pub id: i32,
    pub param_positional: Vec<f64>,
    pub param_keyword: HashMap<String, f64>,
    pub component_type: ComponentType,
}

// const WHITELIST_NONDFA:[&str;2] = ["MP2", "HF"];


impl DFAComponent {
    pub fn new(factor: f64, func: String) -> Self {
        DFAComponent {
            factor,
            func,
            func_full_name: String::new(),
            id: 0,
            param_positional: Vec::new(),
            param_keyword: HashMap::new(),
            component_type: ComponentType::Unknown,
        }
    }

    pub fn formatted_output(&self) -> String {
        // output line by line
        let mut result = format!("component_type: {} factor: {}, func: {}", self.component_type.as_str(), self.factor, self.func);
        if !self.func_full_name.is_empty() {
            result.push_str(&format!(", func_full_name: {}", self.func_full_name));
        }
        if self.id != 0 {
            result.push_str(&format!(", id: {}", self.id));
        }
        if !self.param_positional.is_empty() {
            result.push_str(&format!(", param_positional: {:?}", self.param_positional));
        }
        if !self.param_keyword.is_empty() {
            result.push_str(&format!(", param_keyword: {:?}", self.param_keyword));
        }
        result
    }

    pub fn has_parameter(&self) -> bool {
        !self.param_keyword.is_empty() || !self.param_positional.is_empty()
    }

    pub fn is_unknown(&self) -> bool {
        self.component_type == ComponentType::Unknown
    }

    pub fn to_valid_name(&mut self, functype: &str) -> &mut Self {
        // check whitelist
        if WHITELIST_NONLIBXC.contains_key(self.func.as_str()) {
            self.component_type = WHITELIST_NONLIBXC.get(self.func.as_str()).unwrap().clone();
            return self;
        }
        // check pre-defined codes
        if CODES.contains_key(self.func.as_str()) {
            let code = CODES.get(self.func.as_str()).unwrap();
            self.func_full_name = get_name(*code);
            self.id = *code;
            self.component_type = ComponentType::Libxc;
            return self;
        }

        // search libxc full name
        // let prefix = format!("{}_", functype);
        let prefix = format!("{}_", functype);
        let illegal_prefix = ILLEGAL_SHORT_PREFIX.get(functype).unwrap();
        let (possible_complete_prefix, illegal_complete_prefix) = get_possible_prefix(&functype);
        // condition 1
        // check if self.func starts with any of the possible_complete_prefix
        // let mut full_name = String::new();
        for p in possible_complete_prefix.iter() {
            if self.func.starts_with(p) {
                self.func_full_name = self.func.clone();
                if let Some(id) = AVAIL_FUNC.get(&self.func_full_name) {
                    self.id = *id;
                    self.component_type = ComponentType::Libxc;
                } else {
                    panic!("Error: functional {} not found in libxc", self.func_full_name);
                }
                break;
            } 
        }
        for p in illegal_complete_prefix.iter() {
            if self.func.starts_with(p) {
                panic!("Error: functional {} has illegal prefix for type {}", self.func, functype);
            }
        }
        // end of condition 1

        // condition 2,3
        // check if self.func starts with any of the illegal_prefix
        for p in illegal_prefix.iter() {
            if self.func.starts_with(p) {
                panic!("Error: functional {} has illegal prefix for type {}", self.func, functype);
            }
        }
        // if self.func starts with prefix, remove it
        let mut tmp_func = self.func.clone();
        if self.func.starts_with(&prefix) {
            tmp_func = self.func.trim_start_matches(&prefix).to_string();
        }
        // generate all possible full names
        let possible_full_names: Vec<String> = possible_complete_prefix.iter().map(|p| {
            format!("{}{}", p, tmp_func)
        }).collect();
        // search in AVAIL_FUNC
        let matches: Vec<_> = possible_full_names.into_iter()
            .filter(|name| AVAIL_FUNC.contains_key(name))
            .collect();

        if matches.is_empty() {
            // panic!("Error: functional {} not found in libxc", tmp_func);
            // do nothing, leave id as 0
        } else if matches.len() == 1 {
            self.func_full_name = matches[0].clone();
            self.id = *AVAIL_FUNC.get(&self.func_full_name).unwrap();
            self.component_type = ComponentType::Libxc;
        } else {
            panic!("Error: functional {} is ambiguous, possible matches: {:?}", tmp_func, matches);
        }
        // end of condition 2,3
        
        return self;
    }
    
}

lazy_static! {
    static ref POSSIBLE_PREFIX: HashMap<&'static str, Vec<&'static str>> = HashMap::from([
        ("X", vec!["LDA_X_", "GGA_X_", "MGGA_X_", "HYB_GGA_X_", "HYB_MGGA_X_"]),
        ("C", vec!["LDA_C_", "GGA_C_", "MGGA_C_"]),
        ("XC", vec!["LDA_XC_", "GGA_XC_", "MGGA_XC_", "HYB_LDA_XC_", "HYB_GGA_XC_", "HYB_MGGA_XC_"]),
    ]);
    static ref ILLEGAL_SHORT_PREFIX: HashMap<&'static str, Vec<&'static str>> = HashMap::from([
        ("X", vec!["C_", "XC_"]),
        ("C", vec!["X_", "XC_"]),
        ("XC", vec!["X_", "C_"]),
        ("any", vec![]),
    ]);
}

pub fn get_possible_prefix(functype: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let mut possible_prefix = Vec::new();
    let mut illegal_prefix = Vec::new();
    if functype == "X" {
        possible_prefix = POSSIBLE_PREFIX.get("X").unwrap().clone();
        illegal_prefix = POSSIBLE_PREFIX.get("C").unwrap().clone();
        illegal_prefix.extend(POSSIBLE_PREFIX.get("XC").unwrap().clone());
    } else if functype == "C" {
        possible_prefix = POSSIBLE_PREFIX.get("C").unwrap().clone();
        illegal_prefix = POSSIBLE_PREFIX.get("X").unwrap().clone();
        illegal_prefix.extend(POSSIBLE_PREFIX.get("XC").unwrap().clone());
    } else if functype == "XC" {
        possible_prefix = POSSIBLE_PREFIX.get("XC").unwrap().clone();
        illegal_prefix = POSSIBLE_PREFIX.get("X").unwrap().clone();
        illegal_prefix.extend(POSSIBLE_PREFIX.get("C").unwrap().clone());
    } else if functype == "any" {
        possible_prefix = POSSIBLE_PREFIX.get("X").unwrap().clone();
        possible_prefix.extend(POSSIBLE_PREFIX.get("C").unwrap().clone());
        possible_prefix.extend(POSSIBLE_PREFIX.get("XC").unwrap().clone());
    } else {
        panic!("Error: unknown functype {}", functype);
    }
    (possible_prefix, illegal_prefix)
}

pub fn check_type_sanity(name: &str, functype: &str) -> bool {
    let mut sanity = true;
    let parts: Vec<&str> = name.split(',').collect();
    let n_part_notempty = parts.iter().filter(|p| !p.trim().is_empty()).count();
    if (functype == "X" || functype == "C") && n_part_notempty > 1 {
        sanity =  false;
    }
    sanity
}


pub fn parse_tokens(mut components: Vec<DFAComponent>, functype: &str) -> Vec<DFAComponent> {
    let mut result = Vec::new();
    for comp in components.iter_mut() {
        comp.to_valid_name(functype);
        
        if !comp.is_unknown() {
            result.push(comp.clone());
        } else if ALIAS.contains_key(comp.func.as_str()) {
            if comp.has_parameter() {
                panic!("Error: functional {} has parameters, cannot be filtered by alias", comp.func);
            }
            let alias_xc = ALIAS.get(comp.func.as_str()).unwrap();
            // check if X func is aliased to XC
            if !check_type_sanity(alias_xc, functype) {
                panic!("Error: functional {} is aliased to a different type, which is not allowed in type {}", comp.func, functype);
            }
            let alias_components = parse_1step(alias_xc);
            for mut alias_comp in alias_components {
                alias_comp.factor *= comp.factor;
                result.push(alias_comp);
            }   
        } else {
            // result.push(comp.clone());
            panic!("Error: functional {} not found in libxc and not in alias/whitelist", comp.func);
        }
    }
    // for comp in result.iter_mut() {
    //     comp.to_valid_name(functype);
    // }
    result
}

pub enum ParsedResult {
    VecDFAComponent(Vec<DFAComponent>),
    DFA2step(DFA2step),
}

pub struct DFA2step {
    pub xc_scf: Vec<DFAComponent>,
    pub xc: Vec<DFAComponent>,
    pub reference: String,
}

pub fn parse(xc: &str) -> ParsedResult {
    println!("Parsing xc: {}", xc);
    // check MULTISTEP
    if MULTISTEP.contains_key(xc) {
        let steps = MULTISTEP.get(xc).unwrap();
        println!("Detected multi-step functional {}, which is parsed to:", xc);
        println!("Step for SCF         : {}", steps.code_scf);
        println!("Step for final energy: {}", steps.code);
        println!("Reference: {}", steps.reference);
        let final_components_scf = parse_1step(&steps.code_scf);
        println!("Components for SCF:");
        final_components_scf.iter().for_each(|c| {
            println!("{}", c.formatted_output());
        });
        let final_components = parse_1step(&steps.code);
        println!("Components for final energy:");
        final_components.iter().for_each(|c| {
            println!("{}", c.formatted_output());
        });
        let dfa_steps = DFA2step {
            xc_scf: final_components_scf,
            xc: final_components,
            reference: steps.reference.clone(),
        };
        return ParsedResult::DFA2step(dfa_steps);
    } else {
        let final_components = parse_1step(xc);
        final_components.iter().for_each(|c| {
            println!("{}", c.formatted_output());
        });
        return ParsedResult::VecDFAComponent(final_components);
    }

}

pub fn parse_1step(xc: &str) -> Vec<DFAComponent> {
    // let available_functionals = get_available_functionals();
    let (xc_pass1, captures) = parse_pass1(xc);
    let parts = parse_pass2(&xc_pass1);
    let mut final_components:Vec<DFAComponent> = Vec::new();
    if parts.len() == 2 {
        let (xfac, xfuncs, xparams) = parse_pass3(parts[0], &captures);
        let mut x_components:Vec<DFAComponent> = to_dfa_component_raw(xfac, xfuncs, xparams);
        x_components = parse_tokens(x_components, "X");
        
        let (cfac, cfuncs, cparams) = parse_pass3(parts[1], &captures);
        let mut c_components:Vec<DFAComponent> = to_dfa_component_raw(cfac, cfuncs, cparams);
        c_components = parse_tokens(c_components, "C");

        // x_components.iter().for_each(|c| {
        //     println!("X component: {}", c.formatted_output());
        // });
        // c_components.iter().for_each(|c| {
        //     println!("C component: {}", c.formatted_output());
        // });
        final_components.extend(x_components);
        final_components.extend(c_components);
    } else {
        let (xcfac, xcfuncs, xcparams) = parse_pass3(parts[0], &captures);
        let mut xc_components:Vec<DFAComponent> = to_dfa_component_raw(xcfac, xcfuncs, xcparams);
        xc_components = parse_tokens(xc_components, "XC");
        // xc_components.iter().for_each(|c| {
        //     println!("XC component: {}", c.formatted_output());
        // });
        final_components.extend(xc_components);
    }
    final_components
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
    let re = regex::Regex::new(r"([+-]?\s?\d*\.?\d*)\s?\*?\s?([A-Za-z0-9_:]+)").unwrap();
    for cap in re.captures_iter(xc) {
        //remove whitespace in cap[1]
        let cap1 = cap[1].replace(" ", "");
        let factor = if &cap1 == "" || &cap1 == "+" {
            1.0
        } else if &cap1 == "-" {
            -1.0
        } else {
            cap1.parse::<f64>().unwrap()
        };
        fac.push(factor);
        funcs.push(cap[2].to_string().to_uppercase());
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
        let mut component = DFAComponent::new(fac[i], funcs[i].clone());
        component.param_keyword = param_keyword;
        component.param_positional = param_positional;
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

#[test]
fn test_parse_xc_param() {
    let input1 = "0.5*PBE + 0.5*B88, PBE(_beta=0.1)";
    let final_components = parse_1step(input1);
    assert_eq!(final_components.len(), 3);
    assert_eq!(final_components[0].factor, 0.5);
    assert_eq!(final_components[0].id, 101);
    assert_eq!(final_components[1].id, 106);
    assert_eq!(final_components[2].factor, 1.0);
    assert_eq!(final_components[2].id, 130);
    assert!(final_components[2].param_keyword.get("_beta").unwrap() - 0.1 < 1e-9);
}

#[test]
fn test_parse_xc_hybrid() {
    let input1 = ".2*HF + 0.08*LDA + 0.72*B88, 0.81*LYP + 0.19*VWN3";
    let final_components = parse_1step(input1);
    assert_eq!(final_components.len(), 5);
    assert_eq!(final_components[0].component_type, ComponentType::HF);
    assert_eq!(final_components[0].factor, 0.2);
    assert_eq!(final_components[1].id, 1);
    assert_eq!(final_components[2].id, 106);
    assert_eq!(final_components[3].id, 131);
    assert_eq!(final_components[4].id, 8);
}

#[test]
fn test_pass3_lincomb() {
    let input = "K1 + 0.5*K2 - 0.5*K3 + K4 - K5 +0.111*K6 -21*K_7";
    let param_captures = vec![];
    let (fac, funcs, _params) = parse_pass3(input, &param_captures);
    assert_eq!(fac, vec![1.0, 0.5, -0.5, 1.0, -1.0, 0.111, -21.0]);
    assert_eq!(funcs, vec!["K1", "K2", "K3", "K4", "K5", "K6", "K_7"]);
}