extern mod extra;
extern mod OpenCL;
extern mod rtlsdr;
extern mod dsputils;
extern mod triangle;

use OpenCL::mem::CLBuffer;
use extra::complex;
use std::io;
use std::io::File;
use std::io::Reader;
use std::str;
use extra::time;

#[link_args = "-lOpenCL"] extern {}

fn main() {


	// rtlsdr config
	let sRate: f32 = 2.048e6;
	let centerFreq: f32 = 433e6;

	let x = 512;
	let y = 160;//((sRate/x as f32)) as uint;
	println!("({}, {})", x, y)
	let blockSize = x*y;

	let devHandle = rtlsdr::openDevice(0);
	rtlsdr::setSampleRate(devHandle, sRate as u32);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGainAuto(devHandle);
	rtlsdr::setFrequency(devHandle, centerFreq as u32);

	// load fft kernel, instantiate context
	let ker = File::open(&std::path::Path::new("./fft.cl")).read_to_end();

    let ker = str::from_utf8(ker);

	let (device, ctx, queue) = OpenCL::util::create_compute_context().unwrap();

	let inBuff: CLBuffer<complex::Complex32> = ctx.create_buffer(blockSize, OpenCL::CL::CL_MEM_WRITE_ONLY);
	let outBuff: CLBuffer<complex::Complex32> = ctx.create_buffer(blockSize, OpenCL::CL::CL_MEM_READ_ONLY);

	let program = ctx.create_program_from_source(ker);

	program.build(&device);

	let kernel = program.create_kernel("fft1D_512");

	kernel.set_arg(0, &inBuff);
	kernel.set_arg(1, &outBuff);

	// build bitmap sink
	let videoChan = triangle::spawnVectorVisualSink(x, y);
	// start reading
	let pdata = rtlsdr::readAsync(devHandle, blockSize as u32);
	let mut acc : f32 = 0.0;
	let mut i : uint = 0;

	'main: loop {
		let start = time::precise_time_ns();
		// read samples
		let samples: ~[complex::Complex32] = rtlsdr::dataToSamples(pdata.recv());
		// queue gpu write
		queue.write(&inBuff, &samples.slice(0, samples.len()), ());
		// fft
		let event = queue.enqueue_async_kernel(&kernel, (x/8u, y), Some((x/8, 1)), ());
		let datafft: ~[complex::Complex32] = queue.get(&outBuff, &event);
		// take magnitude
		let dftF: ~[f32] = datafft.iter().map(|x| {let (m, p) = x.to_polar(); m}).to_owned_vec();
		let &dmax: &f32 = dftF.iter().max().unwrap();
		let d: ~[f32] = dftF.iter().map(|&x: &f32| x/dmax).to_owned_vec();
		// try to send, if you can't send, quit
		if !videoChan.try_send(d){
			println!("average of {} ticks: {} ms, data every {} ms", i, acc/i as f32, 1000.0/(sRate/blockSize as f32));
			break 'main
		}
		let end = time::precise_time_ns();
		i += 1;
		acc += (end-start) as f32 / 1e6
	}

	// stop rtlsdr
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}
