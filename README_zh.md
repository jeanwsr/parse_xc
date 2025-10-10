# parse_xc 规则

## 一级结构

xc String 可以由`,`分割为两个区域，或只有一个区域，不可以有大于两个区域。（`()`内的`,`不计入）

例如 `0.5*PBE + 0.5*B88, PBE` 或 `B3LYP`。

当有两个区域时，左边为 exchange 区域，右边为 correlation 区域。只有一个区域时，不做区分。

## 二级结构
对于每个区域，可以写成泛函 component 的线性组合。每一项中系数在前，component 名称在后，中间用`*`连接。可以没有系数，视为`1.0`。

例如 `PBE - 0.1*B88 + 23.45*PBEsol`。

## 三级结构
对于每个泛函 component，做如下规定。
### 名字

* 可以是 libxc full name。例如`GGA_X_PBE`。但在 exchange 区域不能出现 correlation 类型的名字，反之亦然。
* 可以带前缀 `X_`, `C_`, `XC_`。同样，不能出现和区域不符的前缀。
* 可以不带前缀，例如`PBE`。根据其所处的区域，视为带有 `X_`, `C_` 或 `XC_`前缀。
对于此类和上一类情况，将从所有可能的 full name 中查询匹配的名字。若有多个匹配的结果则报错。

以下是若干示例
```bash
> parse_xc "0.5*PBE + 0.5*B88, PBE(_beta=0.1)"                                     ✔  base  
X component: Factor: 0.5, Func: PBE, Full Name: GGA_X_PBE, ID: 101
X component: Factor: 0.5, Func: B88, Full Name: GGA_X_B88, ID: 106
C component: Factor: 1, Func: PBE, Full Name: GGA_C_PBE, ID: 130, Keyword Params: {"_beta": 0.1}
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
