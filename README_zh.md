# parse_xc 规则

## 一级结构

xc String 可以由`,`分割为两个区域，或只有一个区域，不可以有大于两个区域。（`()`内的`,`不计入）

例如 `0.5*PBE + 0.5*B88, PBE` 或 `B3LYP`。

当有两个区域时，左边为 X(exchange) 区域，右边为 C(correlation) 区域。只有一个区域时，称为 XC 区域。

## 二级结构
对于每个区域，可以写成泛函 component 的线性组合。每一项中系数在前，component 名称在后，中间用`*`连接。可以没有系数，视为`1.0`。

例如 `PBE - 0.1*B88 + 23.45*PBEsol`。

## 三级结构
对于每个泛函 component，做如下规定。
### 名字
按以下优先级解析
1. WHITELIST_NONLIBXC，如 HF,MP2 等。
2. 预定义的 CODES。如 `LDA -> 1, B3LYPG -> 402`。此时会直接得到 libxc name 和 id。
3. ALIAS，递归展开 alias，例如 `PBE -> PBE,PBE`。有两个区域时，X/C 区域的 alias 不能展开成其他类型的泛函，例如
```bash
> parse_xc "BLYP,"
Error: functional BLYP is aliased to a different type, which is not allowed in type X
```
而 `BLYP` 是合法的
```bash
> parse_xc "BLYP"
component_type: Libxc factor: 1, func: B88, func_full_name: GGA_X_B88, id: 106
component_type: Libxc factor: 1, func: LYP, func_full_name: GGA_C_LYP, id: 131
```
4. 其他常规 libxc 泛函
* 可以是 libxc full name。例如`GGA_X_PBE`。但在 exchange 区域不能出现 correlation 类型的名字，反之亦然。
* 可以带前缀 `X_`, `C_`, `XC_`。同样，不能出现和区域不符的前缀。
* 可以不带前缀，例如`PBE`。根据其所处的区域，视为带有 `X_`, `C_` 或 `XC_`前缀。
对于此类和上一类情况，将从所有可能的 full name 中查询匹配的名字。若有多个匹配的结果则报错。

以下是若干示例
```bash
> parse_xc "0.5*PBE + 0.5*B88, PBE(_beta=0.1)"
component_type: Libxc factor: 0.5, func: PBE, func_full_name: GGA_X_PBE, id: 101
component_type: Libxc factor: 0.5, func: B88, func_full_name: GGA_X_B88, id: 106
component_type: Libxc factor: 1, func: PBE, func_full_name: GGA_C_PBE, id: 130, param_keyword: {"_beta": 0.1}
> parse_xc ".2*HF + 0.08*LDA + 0.72*B88, 0.81*LYP + 0.19*VWN3"
component_type: HF factor: 0.2, func: HF
component_type: Libxc factor: 0.08, func: LDA, func_full_name: LDA_X, id: 1
component_type: Libxc factor: 0.72, func: B88, func_full_name: GGA_X_B88, id: 106
component_type: Libxc factor: 0.81, func: LYP, func_full_name: GGA_C_LYP, id: 131
component_type: Libxc factor: 0.19, func: VWN3, func_full_name: LDA_C_VWN_RPA, id: 8
```
不合法的情况
```bash
> parse_xc "0.5*PBE + 0.5*B88, X_PBE" 
Error: functional X_PBE has illegal prefix for type C
```
虽然存在泛函 `LDA_X_YUKAWA`，但是此处声明在 XC 区域而非 X 区域，所以找不到（这个行为与 pyscf 一致）
```bash
> parse_xc "B3LYP + YUKAWA"
Error: functional YUKAWA not found in libxc
```
### 参数
每个 component 可以有参数，置于`()`内。形式只能是 positional 或 keyword 其中的一种（前者为`(0.3,0.4)`，后者为`(a=0.3,b=0.4)`）。


以上就完成了单步泛函的解析。其解析结果为 `Vec<DFAComponent>`。
```rust
pub struct DFAComponent {
    pub factor: f64,
    pub func: String,
    pub func_full_name: String,
    pub id: i32,
    pub param_positional: Vec<f64>,
    pub param_keyword: HashMap<String, f64>,
    pub component_type: ComponentType,
}

pub enum ComponentType {
    HF,
    PT2,
    RPA,
    Disp,
    Libxc,
    Unknown,
}
```

## 杂化泛函
目前已实现了对于显式定义`HF`的杂化泛函的解析，例如上面`".2*HF + 0.08*LDA + 0.72*B88, 0.81*LYP + 0.19*VWN3"`的例子。
此时 `HF` 也是一个独立的 component。
对于普通的杂化泛函，我倾向于也将其解析成 Libxc component 和 HF component，如
```
component_type: Libxc factor: 1, func: B3LYP, func_full_name: HYB_GGA_XC_B3LYP, id: 402
component_type: HF factor: 0.2, func: HF
```
这一点待实现。泛函整体的 global hybrid 将通过检索 HF component 的内容来实现。
这种独立的 HF （可能还有 SR_HF）会有助于定义多个 omega-component 的泛函。
不过缺点是解析结果是 `Vec<DFAComponent>`，它自己显然没有一个 hybrid 参数，而是存在其中一个 component 里面。这会增加一些理解的负担。

欢迎讨论其他的方案。

## 多步泛函
目前采用内置 json （与ajz34/dh类似的格式）的方式支持了一些多步泛函。其解析优先级为最高。即首先会判断是否是多步泛函，然后分别执行多步或一步的解析。

例如
```bash
> parse_xc "XYG3" 
Parsing xc: XYG3
Detected multi-step functional XYG3, which is parsed to:
Step for SCF         : B3LYPg
Step for final energy: 0.8033*HF - 0.0140*LDA + 0.2107*B88, 0.6789*LYP + 0.3211*MP2
Reference: 10.1073/pnas.0901093106
Components for SCF:
component_type: Libxc factor: 1, func: B3LYPG, func_full_name: HYB_GGA_XC_B3LYP, id: 402
Components for final energy:
component_type: HF factor: 0.8033, func: HF
component_type: Libxc factor: -0.014, func: LDA, func_full_name: LDA_X, id: 1
component_type: Libxc factor: 0.2107, func: B88, func_full_name: GGA_X_B88, id: 106
component_type: Libxc factor: 0.6789, func: LYP, func_full_name: GGA_C_LYP, id: 131
component_type: PT2 factor: 0.3211, func: MP2
```

多步泛函的解析返回值为 
```rust
pub struct DFA2step {
    pub xc_scf: Vec<DFAComponent>,
    pub xc: Vec<DFAComponent>,
    pub reference: String,
}
```

关于如何支持其他形式的多步泛函输入，例如 `String`, `Vec<String>`，待议。
需要将全部可能的PT2,RPA component 写入 WHITELIST_NONLIBXC，这一点待实现。