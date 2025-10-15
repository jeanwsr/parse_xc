use core::{panic};
use regex;
use std::collections::HashMap;
use lazy_static::lazy_static;
// mod libxc;
use super::libxc::{XcFuncType,LibXCFamily};
// mod xc_helper;
use super::xc_helper::{AVAIL_FUNC, ALIAS, CODES, WHITELIST_NONLIBXC, NAME_WITH_DASH, NAME_WITHOUT_UNDERSCORE,
    ComponentType, 
    get_name, MULTISTEP};
use assert_float_eq::{assert_f64_near};

#[derive(Clone)]
pub struct DFAComponent {
    pub factor: f64,
    pub func: String,
    pub func_full_name: String,
    pub id: usize,
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
        let mut result = format!("component_type: {}, factor: {}, func: {}", self.component_type.as_str(), self.factor, self.func);
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

    pub fn is_nonlibxc(&self) -> bool {
        // HF, PT2, RPA, SBGE2
        self.component_type != ComponentType::Libxc && self.component_type != ComponentType::Unknown
    }

    pub fn check_whitelist(&mut self, functype: &str) -> &mut Self {
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
        return self;
    }
    pub fn to_valid_name(&mut self, functype: &str) -> &mut Self {
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
        // check if self.func starts with any key in POSSIBLE_PREFIX
        // let mut tmp_func = self.func.clone();
        let mut possible_full_names = Vec::new();
        for (short, prefixes) in POSSIBLE_PREFIX.iter() {
            if self.func.starts_with(short) {
                let current_prefix = format!("{}_", short);
                let cutted_name = self.func.trim_start_matches(&current_prefix);
                for p in prefixes.iter() {
                    let full_name = format!("{}{}", p, cutted_name);
                    possible_full_names.push(full_name);
                }
                break;
            }
        }
        // if not, try to add prefix
        if possible_full_names.is_empty() {
            for p in possible_complete_prefix.iter() {
                let full_name = format!("{}{}", p, self.func);
                possible_full_names.push(full_name);
            }
        }      
        // if self.func.starts_with(&prefix) {
        //     tmp_func = self.func.trim_start_matches(&prefix).to_string();
        // }
        // // generate all possible full names
        // let possible_full_names: Vec<String> = possible_complete_prefix.iter().map(|p| {
        //     format!("{}{}", p, tmp_func)
        // }).collect();
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
            panic!("Error: functional {} is ambiguous, possible matches: {:?}", self.func, matches);
        }
        // end of condition 2,3

        // println!("After to_valid_name: {}", self.formatted_output());
        
        return self;
    }

    pub fn get_hybrid(&self, spin_channel: usize) -> f64 {
        if self.component_type == ComponentType::HF {
            return self.factor;
        } else if self.component_type == ComponentType::Libxc {
            let xcfunc = XcFuncType::xc_func_init(self.id, spin_channel);
            match xcfunc.xc_func_family {
                LibXCFamily::HybridGGA | LibXCFamily::HybridMGGA => {
                    let hyb = xcfunc.xc_hyb_exx_coeff();
                    return self.factor * hyb;
                },
                _ => return 0.0,
            }
        } else {
            return 0.0;
        }
    }

    pub fn get_reference(&self) -> String {
        if self.component_type == ComponentType::Libxc && self.id != 0 {
            todo!();
        } else {
            return String::new();
        }
    }
    
}

trait Addable {
    fn is_addable_with(&self, other: &Self) -> bool;
}

impl Addable for DFAComponent {
    fn is_addable_with(&self, other: &Self) -> bool {
        if self.component_type != other.component_type {
            return false;
        }
        if self.id != other.id {
            return false;
        }
        // todo: more precise check for parameters
        if self.param_keyword != other.param_keyword {
            return false;
        }
        if self.param_positional != other.param_positional {
            return false;
        }
        true
    }
}

impl std::ops::Add for DFAComponent {
    type Output = DFAComponent;

    fn add(self, other: DFAComponent) -> DFAComponent {
        // if !self.is_addable_with(&other) {
        //     panic!("Error: cannot add two different DFAComponents");
        // }
        DFAComponent {
            factor: self.factor + other.factor,
            func: self.func.clone(),
            func_full_name: self.func_full_name.clone(),
            id: self.id,
            param_positional: self.param_positional.clone(),
            param_keyword: self.param_keyword.clone(),
            component_type: self.component_type.clone(),
        }
    }
}

pub struct DFAdef {
    pub xc_scf: Option<Vec<DFAComponent>>,
    pub xc: Vec<DFAComponent>,
    pub reference: Vec<String>,
}

impl DFAdef {
    pub fn formatted_output(&self) -> String {
        let mut result = String::new();
        if let Some(scf_components) = &self.xc_scf {
            result.push_str("SCF components:\n");
            for comp in scf_components {
                result.push_str(&format!("  {}\n", comp.formatted_output()));
            }
        }
        result.push_str("Final energy components:\n");
        for comp in &self.xc {
            result.push_str(&format!("  {}\n", comp.formatted_output()));
        }
        if !self.reference.is_empty() {
            result.push_str("References:\n");
            for r in &self.reference {
                result.push_str(&format!("  {}\n", r));
            }
        }
        result
    }
    
    pub fn get_hybrid(&self, spin_channel: usize) -> f64 {
        // sum up all hybrid components in self.xc
        self.xc.iter().map(|c| c.get_hybrid(spin_channel)).sum()
    }

    pub fn summary(&self) {
        println!("{}", self.formatted_output());
        let hyb_0 = self.get_hybrid(0);
        println!("Total hybrid: {}", hyb_0);
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
        // ("XC", vec!["X_", "C_"]),
        ("XC", vec![]),
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
        // illegal_prefix = POSSIBLE_PREFIX.get("X").unwrap().clone();
        // illegal_prefix.extend(POSSIBLE_PREFIX.get("C").unwrap().clone());
        possible_prefix.extend(POSSIBLE_PREFIX.get("X").unwrap().clone());
        possible_prefix.extend(POSSIBLE_PREFIX.get("C").unwrap().clone());
    } else if functype == "any" {
        possible_prefix = POSSIBLE_PREFIX.get("X").unwrap().clone();
        possible_prefix.extend(POSSIBLE_PREFIX.get("C").unwrap().clone());
        possible_prefix.extend(POSSIBLE_PREFIX.get("XC").unwrap().clone());
    } else {
        panic!("Error: unknown functype {}", functype);
    }
    (possible_prefix, illegal_prefix)
}

// pub fn check_type_sanity(name: &str, functype: &str) -> bool {
//     let mut sanity = true;
//     let parts: Vec<&str> = name.split(',').collect();
//     let n_part_notempty = parts.iter().filter(|p| !p.trim().is_empty()).count();
//     if (functype == "X" || functype == "C") && n_part_notempty > 1 {
//         sanity =  false;
//     }
//     sanity
// }


pub fn parse_tokens(mut components: Vec<DFAComponent>, functype: &str) -> Vec<DFAComponent> {
    let mut result = Vec::new();
    let allow_alias = functype == "XC" || functype == "any";
    for comp in components.iter_mut() {
        // comp.to_valid_name(functype);
        comp.check_whitelist(functype);
        
        if comp.is_nonlibxc() {
            result.push(comp.clone());
        } else if allow_alias && ALIAS.contains_key(comp.func.as_str()) {
            if comp.has_parameter() {
                panic!("Error: functional {} has parameters, cannot be filtered by alias", comp.func);
            }
            let alias_xc = ALIAS.get(comp.func.as_str()).unwrap();
            // check if X func is aliased to XC
            // if !check_type_sanity(alias_xc, functype) {
            //     panic!("Error: functional {} is aliased to a different type, which is not allowed in type {}", comp.func, functype);
            // }
            let alias_components = parse_1step(alias_xc);
            for mut alias_comp in alias_components {
                alias_comp.factor *= comp.factor;
                result.push(alias_comp);
            }   
        } else {
            comp.to_valid_name(functype);
            if !comp.is_unknown() {
                result.push(comp.clone());
            } else {
                panic!("Error: functional {} not found in libxc and not in alias/whitelist", comp.func);
            }
        }
    }
    // for comp in result.iter_mut() {
    //     comp.to_valid_name(functype);
    // }
    result
}

// pub enum ParsedResult {
//     VecDFAComponent(Vec<DFAComponent>),
//     DFA2step(DFA2step),
// }



pub fn parse(xc: &str) -> DFAdef {
    println!("Parsing xc: {}", xc);
    // check MULTISTEP
    if MULTISTEP.contains_key(xc) {
        let steps = MULTISTEP.get(xc).unwrap();
        println!("Detected multi-step functional {}, which is parsed to:", xc);
        println!("Step for SCF         : {}", steps.code_scf);
        println!("Step for final energy: {}", steps.code);
        // println!("Reference: {}", steps.reference);
        let final_components_scf = parse_1step(&steps.code_scf);
        // println!("Components for SCF:");
        // final_components_scf.iter().for_each(|c| {
        //     println!("{}", c.formatted_output());
        // });
        let final_components = parse_1step(&steps.code);
        // println!("Components for final energy:");
        // final_components.iter().for_each(|c| {
        //     println!("{}", c.formatted_output());
        // });
        let mut reference = Vec::new();
        reference.push(steps.reference.clone());
        let dfa_steps = DFAdef {
            xc_scf: Some(final_components_scf),
            xc: final_components,
            reference: reference,
        };
        return dfa_steps;
    } else {
        let mut final_components = parse_1step(xc);
        final_components = merge_components(final_components);
        // final_components.iter().for_each(|c| {
        //     println!("{}", c.formatted_output());
        // });
        let dfa = DFAdef {
            xc_scf: None,
            xc: final_components,
            reference: Vec::new(),
        };
        return dfa;
    }

}

pub fn parse_1step(xc: &str) -> Vec<DFAComponent> {
    // replace dash in name by searching NAME_WITH_DASH
    let mut xc = xc.to_uppercase();
    if xc.contains('-') {
        for (name_with_dash, name_without_dash) in NAME_WITH_DASH.iter() {
            // if xc.contains(name_with_dash) {
            xc = xc.replace(name_with_dash, name_without_dash);
            // }
        }
    }
    // replace non-underscore name by searching NAME_WITHOUT_UNDERSCORE
    for (name_without_underscore, name_with_underscore) in NAME_WITHOUT_UNDERSCORE.iter() {
        if xc.contains(name_without_underscore) {
            xc = xc.replace(name_without_underscore, name_with_underscore);
        }
    }
    let (xc_pass1, captures) = parse_pass1(&xc);
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

pub fn merge_components(components: Vec<DFAComponent>) -> Vec<DFAComponent> {
    let mut merged: Vec<DFAComponent> = Vec::new();
    for comp in components {
        let mut found = false;
        for m in merged.iter_mut() {
            if m.is_addable_with(&comp) {
                *m = m.clone() + comp.clone();
                found = true;
                break;
            }
        }
        if !found {
            merged.push(comp);
        }
    }
    merged
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
        funcs.push(cap[2].to_string()//.to_uppercase()
            );
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
                params.push(param_captures[index - 1].to_lowercase());
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
    assert_f64_near!(final_components[0].factor, 0.5, 9);
    assert_eq!(final_components[0].id, 101);
    assert_eq!(final_components[1].id, 106);
    assert_eq!(final_components[2].factor, 1.0);
    assert_eq!(final_components[2].id, 130);
    assert_f64_near!(final_components[2].param_keyword.get("_beta").unwrap().clone(), 0.1, 9);
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
    let input2 = "B3LYP";
    let final_dfa = parse(input2);
    assert_f64_near!(final_dfa.get_hybrid(0), 0.2, 9);
    let input3 = "0.2*HF + 0.5*B3LYP";
    let final_dfa3 = parse(input3);
    assert_f64_near!(final_dfa3.get_hybrid(0), 0.3, 9);
}

#[test]
fn test_pass3_lincomb() {
    let input = "K1 + 0.5*K2 - 0.5*K3 + K4 - K5 +0.111*K6 -21*K_7";
    let param_captures = vec![];
    let (fac, funcs, _params) = parse_pass3(input, &param_captures);
    assert_eq!(fac, vec![1.0, 0.5, -0.5, 1.0, -1.0, 0.111, -21.0]);
    assert_eq!(funcs, vec!["K1", "K2", "K3", "K4", "K5", "K6", "K_7"]);
}

#[test]
fn test_parse_xc_merge() {
    let input1 = "BLYP + 0.1*X_B88";
    let final_components = parse_1step(input1);
    let merged = merge_components(final_components);
    assert_eq!(merged.len(), 2);
    assert_f64_near!(merged[0].factor, 1.1, 9);
    assert_eq!(merged[0].id, 106);
    let input2 = "BLYP + 0.1*LYP";
    let final_components2 = parse_1step(input2);
    let merged2 = merge_components(final_components2);
    assert_eq!(merged2.len(), 2);
    assert_f64_near!(merged2[1].factor, 1.1, 9);
}

#[test]
fn test_parse_xc_simple() {
    let input1 = "APBE,";
    let final_components = parse_1step(input1);
    assert_eq!(final_components.len(), 1);
    assert_eq!(final_components[0].id, 184);
    let input2 = "LDA0";
    let final_components2 = parse_1step(input2);
    assert_eq!(final_components2[0].id, 177);
    let input3 = "Xpbe,";
    let final_components3 = parse_1step(input3);
    assert_eq!(final_components3[0].id, 123);
    let input4 = "gga_x_pbe_gaussian";
    let final_components4 = parse_1step(input4);
    assert_eq!(final_components4[0].id, 321);
}

#[test]
fn test_parse_xc_dash() {
    let input1 = "M06-L";
    let final_components = parse_1step(input1);
    assert_eq!(final_components.len(), 2);
    assert_eq!(final_components[0].id, 203);
    assert_eq!(final_components[1].id, 233);
    let input2 = "m06-l,m06-2x";
    let final_components2 = parse_1step(input2);
    assert_eq!(final_components2.len(), 2);
}

// mean to fail
#[test]
#[should_panic]
fn test_parse_xc_fail() {
    let input1 = "B3LYP,";
    let _final_components = parse_1step(input1);
}