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
1. WHITELIST_NONDFA，如 MP2 等，待完善
2. ALIAS，递归展开 alias，例如 `PBE -> PBE,PBE`。有两个区域时，X/C 区域的 alias 不能展开成其他类型的泛函，例如
```bash
> parse_xc "BLYP,"
Error: functional BLYP is aliased to a different type, which is not allowed in type X
```
而 `BLYP` 是合法的
```bash
> parse_xc "BLYP"
factor: 1, func: B88, func_full_name: GGA_X_B88, id: 106
factor: 1, func: LYP, func_full_name: GGA_C_LYP, id: 131
```
3. 其他常规 libxc 泛函
* 可以是 libxc full name。例如`GGA_X_PBE`。但在 exchange 区域不能出现 correlation 类型的名字，反之亦然。
* 可以带前缀 `X_`, `C_`, `XC_`。同样，不能出现和区域不符的前缀。
* 可以不带前缀，例如`PBE`。根据其所处的区域，视为带有 `X_`, `C_` 或 `XC_`前缀。
对于此类和上一类情况，将从所有可能的 full name 中查询匹配的名字。若有多个匹配的结果则报错。

以下是若干示例
```bash
> parse_xc "0.5*PBE + 0.5*B88, PBE(_beta=0.1)"
factor: 0.5, func: PBE, func_full_name: GGA_X_PBE, id: 101
factor: 0.5, func: B88, func_full_name: GGA_X_B88, id: 106
factor: 1, func: PBE, func_full_name: GGA_C_PBE, id: 130, param_keyword: {"_beta": 0.1}
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
