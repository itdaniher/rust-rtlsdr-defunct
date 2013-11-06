extern mod extra;
extern mod OpenCL;
extern mod rtlsdr;
extern mod videoSinkSDL2;
extern mod dsputils;

use OpenCL::mem::CLBuffer;
use extra::complex;
use std::rt::io;
use std::rt::io::file;
use std::rt::io::Reader;
use std::str;

fn main() {

	// rtlsdr config
	let sRate: f32 = 2.048e6;
	let centerFreq: f32 = 434e6;
	let blockSize = 1024*640;
	let devHandle = rtlsdr::openDevice(0);
	rtlsdr::setSampleRate(devHandle, sRate as u32);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGainAuto(devHandle);
	rtlsdr::setFrequency(devHandle, centerFreq as u32);

	// load fft kernel, instantiate context
	let ker = file::open(&std::path::Path::new("./fft.cl"), io::Open, io::Read).read_to_end();
    let ker = str::from_utf8(ker);

	let (device, ctx, queue) = OpenCL::util::create_compute_context().unwrap();

	let inBuff: CLBuffer<complex::Complex32> = ctx.create_buffer(blockSize, OpenCL::CL::CL_MEM_WRITE_ONLY);
	let outBuff: CLBuffer<complex::Complex32> = ctx.create_buffer(blockSize, OpenCL::CL::CL_MEM_READ_ONLY);

	let program = ctx.create_program_from_source(ker);

	program.build(&device);

	let kernel = program.create_kernel("fft1D_1024");

	kernel.set_arg(0, &inBuff);
	kernel.set_arg(1, &outBuff);

	// build bitmap sink
	let videoChan = videoSinkSDL2::spawnVectorVisualSink();
	// start reading
	let pdata = rtlsdr::readAsync(devHandle, blockSize as u32);

	'main: loop {
		// read samples
		let samples: ~[complex::Complex32] = rtlsdr::dataToSamples(pdata.recv());
		// queue gpu write
		queue.write(&inBuff, &samples.slice(0, samples.len()), ());
		// fft
		let event = queue.enqueue_async_kernel(&kernel, (1024u/8u, samples.len()/1024), Some((1024/8, 1)), ());
		let datafft: ~[complex::Complex32] = queue.get(&outBuff, &event);
		// take magnitude
		let dftF: ~[f32] = datafft.iter().map(|x| {let (m, p) = x.to_polar(); m}).collect();
		// try to send, if you can't send, quit
		if !videoChan.try_send(dftF){
			break 'main
		}
	}
	// stop rtlsdr
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}
