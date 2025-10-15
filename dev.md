## 结构体用法对照

| DFAdef | DFA4REST |
| -- | -- |
| `get_hybrid()` | `get_hybrid_libxc()` |
| | `init_libxc() -> XcFuncType`|
| | `parse_scf(name: &str, spin_channel: usize)` |
| | `is_hybrid()` |
| | `use_density_gradient()` |