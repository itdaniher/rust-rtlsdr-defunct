extern mod sdl2;
extern mod extra;
use extra::time;
use std::comm;
use std::task;
mod dsputils;

pub fn drawVectorAsBarPlot (renderer: &sdl2::render::Renderer, mut data: ~[f32]){
	// downsample to 800px if needbe
	let (sw, sh) = renderer.get_size();
	let len: uint = data.len() as uint;
	let px: uint = sw as uint;
	data = data.iter().enumerate().filter(|&(x, &y)| (x % (len/px + 1)) == 0).map(|(x, &y)| y).collect();
	// black screen background
	renderer.set_draw_color(sdl2::pixels::RGB(0, 0, 0));
	renderer.clear();
	// calculate bar width
	let width: f32 = sw as f32 / (data.len() as f32);
	let height: f32 = sh as f32;
	// find max value
	let &dmax: &f32 = data.iter().max().unwrap();
	let &dmin: &f32 = data.iter().min().unwrap();
	// calculate height scale value
	let scale: f32 = height / (2f32*(dmax-dmin));
	assert!(width > 1.0);
	data.reverse();
	data.iter().enumerate().map(|(i, &x)| {
		let mut yf = height*0.5f32;
		let mut hf = scale*x;
		if (x > 0f32) {yf -= x*scale;}
		if (x < 0f32) {hf = -1f32*hf;}
		let r = sdl2::rect::Rect (
			((sw as f32)- width*(i as f32 + 1.0)) as i32,
			yf as i32,
			width as i32,
			hf as i32);
		renderer.set_draw_color(sdl2::pixels::RGB(0, 127, 0));
		renderer.fill_rect(r)
	}).len();
}

pub fn doWorkWithPEs (pDataC: comm::Port<~[f32]>) {
	let mut lastDraw: u64 = 0;
	sdl2::init([sdl2::InitVideo]);
	let window =  match sdl2::video::Window::new("rust-sdl2 demo: Video", sdl2::video::PosCentered, sdl2::video::PosCentered, 800, 600, [sdl2::video::OpenGL]) {
		Ok(window) => window,
		Err(err) => fail!("")
	};
	let renderer =  match sdl2::render::Renderer::from_window(window, sdl2::render::DriverAuto, [sdl2::render::Accelerated]){
		Ok(renderer) => renderer,
		Err(err) => fail!("")
	};
	'main : loop {
		if pDataC.peek() {
			let d = pDataC.recv();
			drawVectorAsBarPlot(renderer, d);
		}
		if ((time::precise_time_ns() - lastDraw) > ((1f/30f)*1e9) as u64) {
			lastDraw = time::precise_time_ns();
			renderer.present()
		}
	}
}

pub fn spawnVectorVisualSink() -> (comm::Chan<~[f32]>){
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	let mut t = task::task();
	t.sched_mode(task::SingleThreaded);
	t.spawn_with(pData, doWorkWithPEs);
	return cData;
}

fn main () {
	let c = spawnVectorVisualSink();
	c.send(dsputils::bpf(511, 20.0/881e3, 20e3/881e3));
	loop {}
}
