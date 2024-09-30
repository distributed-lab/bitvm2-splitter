# :jigsaw: Bitcoin Splitter

This is a crate for splitting the Bitcoin script into multiple parts as suggested by the recent [_BitVM2 paper_](https://bitvm.org/bitvm_bridge.pdf).

## :raising_hand: But What is Bitcoin Splitter?

Suppose we have the input $x$ and the function $f$ and the prover wants to convince the skeptical verifier that $y=f(x)$. Of course, one way to do that is to publish the following Bitcoin script:

```bash
<x> <f> <y> OP_EQUAL
```

However, the main issue is that besides basic usages such as hash function computation (in that case, $f=H$ for native _SHA-256_ hash function), typically $f$ is very large. The BitVM2 paper suggests splitting the function $f$ into multiple parts and publishing them separately. Formally, suppose

$$
f = f_n \circ f_{n-1} \circ \cdots \circ f_1
$$

Then, instead of proving $y=f(x)$, the prover can prove the following statements:

$$
z_1 = f_1(z_0), \quad z_2 = f_2(z_1), \quad \ldots, \quad z_n = f_n(z_{n-1})
$$

for $z_1:=x,z_n:=y$. Then, the prover publishes $z_1,\dots,z_n$ together with $f_1,\dots,f_n$. Then, the verifier can ensure that all $z_1,\dots,z_n$ were obtained correctly. In case something was computed wrong, the verifier can challenge the prover and claim the bounty. For more details, see _BitVM2 paper_.
