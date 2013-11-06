extern mod sdl2;
extern mod extra;
extern mod dsputils;

use extra::time;
use std::comm;
use std::task;
use std::num;

pub fn doWorkWithPEs (pDataC: comm::Port<~[f32]>) {
	let mut lastDraw: u64 = 0;
	sdl2::init([sdl2::InitVideo]);
	let window =  sdl2::video::Window::new("rust-sdl2 waterfall", sdl2::video::PosCentered, sdl2::video::PosCentered, 1024, 640, []).unwrap();
	let surf = window.get_surface().unwrap();
	do surf.with_lock |pixels| {
		println!("{}", pixels.len());
	}
	'main : loop {
		match sdl2::event::poll_event() {
			sdl2::event::QuitEvent(_) => break 'main,
			_ => {}
		}
		if pDataC.peek() {
			let start = time::precise_time_ns();
			let d = pDataC.recv();
			let &dmax: &f32 = d.iter().max().unwrap();
			let dmaxNormed = 255f32/dmax;
			let d: ~[u8] = d.iter().map(|&x: &f32| (x*dmaxNormed) as u8).collect();
			do surf.with_lock |pixels| {
				for i in range(0u, d.len()) {
					pixels[i*4+1] = d[i];
				}
			}
			let end = time::precise_time_ns();
			println!("{}", (end-start) as f32/1e9);
			if ((time::precise_time_ns() - lastDraw) > ((1f32/30f32)*1e9) as u64) {
				println!("{}", (time::precise_time_ns() - lastDraw)/1000)
				lastDraw = time::precise_time_ns();
				window.update_surface();
			}
		}
	}
	sdl2::quit();
}

pub fn spawnVectorVisualSink() -> (comm::Chan<~[f32]>){
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	let t = task::task().spawn_with(pData, doWorkWithPEs);
	return cData;
}
