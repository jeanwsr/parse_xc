## 结构体用法对照

| DFAdef | DFA4REST |
| -- | -- |
| `scf_xcfunc_iter() -> impl Iterator<Item = XcFuncType> + '_` | `init_libxc() -> XcFuncType`|
| `scf_xcfunc_iter() -> impl Iterator<Item = XcFuncType> + '_`| |
| `dft::parse::parse(xc: &str) -> DFAdef` | |
| `dft::parse::parse_and_derive(xc: &str, ...) -> DFAdef` | `new(name: &str, ...) -> DFA4REST` |
| `get_hybrid_scf() -> f64` | `get_hybrid_libxc() -> f64` |
| `is_hybrid()` | `is_hybrid()` |
| `is_fifth_dfa()` | `is_fifth_dfa()` | 
| `use_density_gradient()` | `use_density_gradient()` |