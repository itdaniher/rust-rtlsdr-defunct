extern mod sdl;
extern mod extra;
use extra::time;
use std::comm;
mod dsputils;

pub fn drawVectorAsBarPlot (screen: &sdl::video::Surface, mut data: ~[f32]){
	// downsample to 800px if needbe
	let len: uint = data.len() as uint;
	let px: uint = screen.get_width() as uint;
	data = data.iter().enumerate().filter(|&(x, &y)| (x % (len/px + 1)) == 0).map(|(x, &y)| y).collect();
	// black screen background
	screen.fill_rect(Some(sdl::Rect {x: 0 as i16, y: 0 as i16, w: screen.get_width(), h: screen.get_height()}), sdl::video::RGB(0,0,0));
	// calculate bar width
	let width: f32 = screen.get_width() as f32 / (data.len() as f32);
	let height: f32 = screen.get_height() as f32;
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
		let r = sdl::Rect {
			x: ((screen.get_width() as f32)- width*(i as f32 + 1.0)) as i16,
			y: yf as i16,
			w: (width) as u16,
			h: hf as u16};
		screen.fill_rect(Some(r), sdl::video::RGB(0,127,0));
	}).len();
}

fn doWorkWithPEs (pDataC: comm::Port<~[f32]>, cUserC: comm::Chan<sdl::event::Key>) {
	do sdl::start {
		let mut lastDraw: u64 = 0;
		sdl::init([sdl::InitVideo]);
		sdl::wm::set_caption("rust-sdl", "rust-sdl");
		let screen = match sdl::video::set_video_mode(1100, 400, 32, [sdl::video::HWSurface], [sdl::video::DoubleBuf]) {
			Ok(screen) => screen,
			Err(err) => fail!(fmt!("failed to set video mode: %s", err))
		};
		'main : loop {
			if pDataC.peek() {
				let d = pDataC.recv();
				drawVectorAsBarPlot(screen, d);
			}
			'event : loop {
				let ev = sdl::event::poll_event();
				match ev {
					sdl::event::QuitEvent => break 'main,
					sdl::event::NoEvent => {break 'event},
					sdl::event::KeyEvent (a,b,c,d) => {if (b == true) {cUserC.send(a)}},
					_ => {println(fmt!("%?", ev));}
				}
			}
			if ((time::precise_time_ns() - lastDraw) > ((1f/30f)*1e9) as u64) {
				lastDraw = time::precise_time_ns();
				screen.flip();
			}
		}
		sdl::quit();
	}
}

pub fn spawnVectorVisualSink() -> (comm::Port<sdl::event::Key>, comm::Chan<~[f32]>){
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::stream();
	let (pUser, cUser): (comm::Port<sdl::event::Key>, comm::Chan<sdl::event::Key>) = comm::stream();
	doWorkWithPEs(pData, cUser);
	return (pUser, cData);
}

fn main () {
	let (p, c) = spawnVectorVisualSink();
	c.send(dsputils::bpf(511, 20.0/881e3, 20e3/881e3));
	loop {}
}
