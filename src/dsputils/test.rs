extern mod extra;
extern mod dsputils;

#[test]
fn testLPF() {
	println(fmt!("%?", dsputils::lpf(511, 20.0e3/88.1e3)));
}
