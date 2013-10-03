extern mod dsputils;
extern mod videoSinkSDL2;

#[test]
fn testDraw() -> () {
	let c = videoSinkSDL2::spawnVectorVisualSink();
	c.send(dsputils::bpf(511, 20.0/881e3, 20e3/881e3));
}
